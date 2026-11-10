//! Explicitly enabled hardware acceptance runner, excluded from normal release builds.
use super::*;
use crate::{
    device::{Scope, targets},
    protocol::PowerCommand,
};
use std::io::Write;

fn acceptance_ids() -> Result<[String; 3]> {
    let value = std::env::var("BASESTATION_ACCEPTANCE_IDS")
        .context("Set BASESTATION_ACCEPTANCE_IDS to three comma-separated station IDs")?;
    let ids: [String; 3] = value
        .split(',')
        .map(|id| id.trim().to_owned())
        .collect::<Vec<_>>()
        .try_into()
        .map_err(|_| anyhow::anyhow!("Expected exactly three station IDs"))?;
    if ids.iter().any(String::is_empty)
        || ids[0] == ids[1]
        || ids[1] == ids[2]
        || ids[0] == ids[2]
    {
        bail!("Expected three distinct, non-empty station IDs");
    }
    Ok(ids)
}

pub fn run() -> Result<()> {
    let station_ids = acceptance_ids()?;
    let ids = station_ids.each_ref().map(String::as_str);
    let dir = crate::config::directory();
    std::fs::create_dir_all(&dir)?;
    let file = std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(dir.join("hardware-acceptance.log"))?;
    let (events, receiver) = std::sync::mpsc::channel();
    let writer = std::thread::spawn(move || -> std::io::Result<()> {
        let mut file = file;
        for event in receiver {
            if let Event::Log(level, message) = event {
                writeln!(
                    file,
                    "{} [{level}] {message}",
                    chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f")
                )?;
                file.flush()?;
            }
        }
        file.sync_all()
    });
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    let result = runtime.block_on(async {
        let mut backend = Backend {
            adapter: None,
            peripherals: BTreeMap::new(),
            devices: BTreeMap::new(),
            events,
        };
        let result = exercise(&mut backend, &dir, ids).await;
        backend.log(
            if result.is_ok() { "Info" } else { "Error" },
            format!("ACCEPTANCE RESULT: {result:?}"),
        );
        backend.disconnect().await;
        result
    });
    writer
        .join()
        .map_err(|_| anyhow::anyhow!("Log writer panicked"))??;
    std::fs::write(dir.join("result.txt"), format!("{result:#?}"))?;
    result
}

async fn exercise(backend: &mut Backend, dir: &std::path::Path, ids: [&str; 3]) -> Result<()> {
    backend.scan().await?;
    // Do not touch additional devices that happen to be nearby.
    backend.devices.retain(|id, _| ids.contains(&id.as_str()));
    let mut baseline = Vec::new();
    for id in ids {
        backend.perform(id, Action::Refresh).await?;
        let device = &backend.devices[id];
        let power = match device.status {
            Status::Sleep => PowerCommand::Sleep,
            Status::Standby => PowerCommand::Standby,
            Status::On(_) => PowerCommand::On,
            _ => bail!("Cannot safely restore {id}: unknown initial power state"),
        };
        let channel = Channel::new(device.channel.context("Missing initial channel")?)?;
        baseline.push((id.to_owned(), power, channel.byte()));
    }
    // Durable journal is written before the first write.
    std::fs::write(
        dir.join("baseline.json"),
        serde_json::to_vec_pretty(&baseline)?,
    )?;
    backend.log("Info", format!("BASELINE: {baseline:?}"));
    let tested: Result<()> = async {
        let selected = std::collections::BTreeSet::from([ids[0].to_owned(), ids[1].to_owned()]);
        for scope in [Scope::One(ids[0]), Scope::Selected(&selected), Scope::All] {
            let targets = targets(&backend.devices, scope);
            for command in [PowerCommand::On, PowerCommand::Standby, PowerCommand::Sleep] {
                let before = snapshot(backend, ids).await?;
                backend.log(
                    "Info",
                    format!("TEST {command:?}; exact targets: {targets:?}"),
                );
                let mut errors = Vec::new();
                for id in &targets {
                    if let Err(error) = backend.perform(id, Action::Power(command)).await {
                        errors.push(format!("{id}: {error:#}"));
                    }
                }
                let after = snapshot(backend, ids).await?;
                for id in ids {
                    if targets.iter().any(|target| target == id) {
                        if !after[id].0.matches(command) {
                            errors.push(format!("{id}: requested state not observed"));
                        }
                    } else if before[id] != after[id] {
                        errors.push(format!("Non-target changed: {id}"));
                    }
                    if before[id].1 != after[id].1 {
                        errors.push(format!("Power changed channel: {id}"));
                    }
                }
                if !errors.is_empty() {
                    bail!("{}", errors.join("; "));
                }
                backend.log(
                    "Info",
                    format!("PASS {command:?}; targets and unaffected devices verified"),
                );
            }
        }
        // All stations are sleeping during temporary channel collisions.
        for value in [1, 5, 16] {
            let before = snapshot(backend, ids).await?;
            backend
                .perform(ids[0], Action::Channel(Channel::new(value)?))
                .await?;
            let after = snapshot(backend, ids).await?;
            if after[ids[0]].1 != Some(value) {
                bail!("Channel {value} not confirmed");
            }
            for id in &ids[1..] {
                if before[*id] != after[*id] {
                    bail!("Channel operation affected non-target {id}");
                }
            }
            backend.log(
                "Info",
                format!("PASS channel {value}; other stations unchanged"),
            );
        }
        Ok(())
    }
    .await;
    // Always attempt every restoration, even after a test or individual restore fails.
    let mut failures = Vec::new();
    backend.log("Info", "RESTORE baseline".into());
    for (id, _, channel) in &baseline {
        if backend.devices[id].channel != Some(*channel)
            && let Err(error) = backend
                .perform(id, Action::Channel(Channel::new(*channel)?))
                .await
        {
            failures.push(format!("Restore channel {id}: {error:#}"));
        }
    }
    for (id, command, _) in &baseline {
        if let Err(error) = backend.perform(id, Action::Power(*command)).await {
            failures.push(format!("Restore power {id}: {error:#}"));
        }
    }
    for (id, command, channel) in &baseline {
        match backend.perform(id, Action::Refresh).await {
            Ok(())
                if backend.devices[id].status.matches(*command)
                    && backend.devices[id].channel == Some(*channel) =>
            {
                backend.log(
                    "Info",
                    format!("RESTORED {id}: {command:?}, channel {channel}"),
                );
            }
            other => failures.push(format!("Baseline verification failed {id}: {other:?}")),
        }
    }
    if !failures.is_empty() {
        bail!(
            "Test: {tested:?}; RESTORATION FAILURE: {}",
            failures.join("; ")
        );
    }
    tested
}

async fn snapshot(
    backend: &mut Backend,
    ids: [&str; 3],
) -> Result<BTreeMap<String, (Status, Option<u8>)>> {
    let mut result = BTreeMap::new();
    for id in ids {
        backend.perform(id, Action::Refresh).await?;
        let d = &backend.devices[id];
        result.insert(id.to_owned(), (d.status.clone(), d.channel));
    }
    Ok(result)
}
