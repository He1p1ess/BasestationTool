use crate::{
    device::{Action, Device, KnownDevice},
    diagnostics,
    protocol::{self, Channel, Status},
};
use anyhow::{Context, Result, bail};
use btleplug::{
    api::{
        Central, CentralEvent, CharPropFlags, Characteristic, Manager as _, Peripheral as _,
        ScanFilter, WriteType,
    },
    platform::{Adapter, Manager, Peripheral},
};
use futures_util::StreamExt;
use std::{collections::BTreeMap, time::Duration};
use tokio::sync::mpsc;

#[cfg(feature = "hardware-tests")]
pub mod acceptance;

pub enum Request {
    Scan,
    Reset,
    Run(Vec<String>, Action),
}
pub enum Event {
    Device(Device),
    Log(&'static str, String),
    Busy(bool),
}
pub struct Worker {
    pub tx: mpsc::Sender<Request>,
    pub rx: std::sync::mpsc::Receiver<Event>,
    thread: Option<std::thread::JoinHandle<()>>,
    stop: tokio::sync::watch::Sender<bool>,
}
impl Worker {
    pub fn start() -> Self {
        let (tx, mut requests) = mpsc::channel(32);
        let (events, rx) = std::sync::mpsc::channel();
        let (stop, mut stopped) = tokio::sync::watch::channel(false);
        let thread =
            std::thread::spawn(move || {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("Tokio runtime");
                rt.block_on(async move {
                let mut backend = Backend {
                    adapter: None,
                    peripherals: BTreeMap::new(),
                    devices: BTreeMap::new(),
                    events,
                };
                loop {
                    let request = tokio::select! {
                        _ = stopped.changed() => break,
                        request = requests.recv() => match request { Some(r) => r, None => break },
                    };
                    let _ = backend.events.send(Event::Busy(true));
                    let work = async { match request {
                        Request::Scan => {
                            if let Err(e) = backend.scan().await {
                                backend.log("Error", format!("Scan: {e:#}"));
                            }
                        }
                        Request::Reset => {
                            backend.disconnect().await;
                            backend.adapter = None;
                            backend.peripherals.clear();
                            if let Err(e) = backend.scan().await {
                                backend.log("Error", format!("Reset: {e:#}"));
                            }
                        }
                        Request::Run(ids, action) => {
                            backend
                                .log("Info", format!("Group {action:?}: {} device(s)", ids.len()));
                            for id in ids {
                                backend.run(&id, action).await;
                            }
                        }
                    }};
                    tokio::select! { _ = stopped.changed() => break, _ = work => {} }
                    let _ = backend.events.send(Event::Busy(false));
                }
                let _ = tokio::time::timeout(Duration::from_secs(2),backend.disconnect()).await;
            });
            });
        Self {
            tx,
            rx,
            thread: Some(thread),
            stop,
        }
    }
}
impl Drop for Worker {
    fn drop(&mut self) {
        let _ = self.stop.send(true);
        if let Some(t) = self.thread.take() {
            let deadline = std::time::Instant::now() + Duration::from_millis(2500);
            while !t.is_finished() && std::time::Instant::now() < deadline {
                std::thread::sleep(Duration::from_millis(20));
            }
            if t.is_finished() {
                let _ = t.join();
            }
        }
    }
}
struct Backend {
    adapter: Option<Adapter>,
    peripherals: BTreeMap<String, Peripheral>,
    devices: BTreeMap<String, Device>,
    events: std::sync::mpsc::Sender<Event>,
}
async fn bounded<T>(f: impl std::future::Future<Output = btleplug::Result<T>>) -> Result<T> {
    tokio::time::timeout(Duration::from_secs(12), f)
        .await
        .context("Bluetooth timeout (12s)")?
        .map_err(Into::into)
}
impl Backend {
    fn log(&self, level: &'static str, message: String) {
        let _ = self.events.send(Event::Log(level, message));
    }
    fn publish(&self, id: &str) {
        if let Some(d) = self.devices.get(id) {
            let _ = self.events.send(Event::Device(d.clone()));
        }
    }
    async fn disconnect(&mut self) {
        if let Some(a) = &self.adapter {
            let _ = bounded(a.stop_scan()).await;
        }
        for (id, p) in &self.peripherals {
            let _ = bounded(p.disconnect()).await;
            if let Some(d) = self.devices.get_mut(id) {
                d.connected = false;
                d.online = false;
            }
            self.publish(id);
        }
    }
    async fn scan(&mut self) -> Result<()> {
        if self.adapter.is_none() {
            let manager = bounded(Manager::new()).await?;
            self.adapter = bounded(manager.adapters()).await?.into_iter().next();
        }
        let adapter = self
            .adapter
            .clone()
            .context("No Bluetooth adapter / Kein Bluetooth-Adapter")?;
        for d in self.devices.values_mut() {
            d.online = false;
            // Retain the explicitly timestamped last advertisement RSSI.
        }
        self.log("Info", "BLE scan started (8s)".into());
        let mut advertisements = bounded(adapter.events()).await?;
        bounded(adapter.start_scan(ScanFilter::default())).await?;
        let result: Result<()> = async {
            let deadline = tokio::time::Instant::now() + Duration::from_secs(8);
            while let Ok(Some(event)) =
                tokio::time::timeout_at(deadline, advertisements.next()).await
            {
                let peripheral_id = match event {
                    CentralEvent::DeviceDiscovered(id)
                    | CentralEvent::DeviceUpdated(id)
                    | CentralEvent::ManufacturerDataAdvertisement { id, .. }
                    | CentralEvent::ServiceDataAdvertisement { id, .. }
                    | CentralEvent::ServicesAdvertisement { id, .. } => id,
                    _ => continue,
                };
                {
                    let p = match bounded(adapter.peripheral(&peripheral_id)).await {
                        Ok(p) => p,
                        Err(e) => {
                            self.log("Warning", format!("Advertisement: {e}"));
                            continue;
                        }
                    };
                    let Some(props) = bounded(p.properties()).await.unwrap_or(None) else {
                        continue;
                    };
                    let name = props.local_name.unwrap_or_default();
                    if !name.starts_with("LHB-") && !props.services.contains(&protocol::SERVICE) {
                        continue;
                    }
                    let id = format!("{}:{}", std::env::consts::OS, p.id());
                    let d = self.devices.entry(id.clone()).or_default();
                    d.known = KnownDevice {
                        id: id.clone(),
                        name,
                        address: props.address.to_string(),
                        alias: String::new(),
                    };
                    d.rssi = props.rssi;
                    d.seen = Some(chrono::Local::now().format("%H:%M:%S").to_string());
                    d.online = true;
                    self.peripherals.insert(id.clone(), p);
                    self.publish(&id);
                }
            }
            Ok(())
        }
        .await;
        let stopped = bounded(adapter.stop_scan()).await;
        result?;
        stopped?;
        // Connected stations may stop advertising. Keep them in the readback set
        // rather than declaring them offline solely because no advertisement arrived.
        for (id, peripheral) in &self.peripherals {
            if bounded(peripheral.is_connected()).await.unwrap_or(false)
                && let Some(device) = self.devices.get_mut(id)
            {
                device.online = true;
            }
        }
        for id in self.devices.keys() {
            self.publish(id);
        }
        let ids: Vec<_> = self
            .devices
            .iter()
            .filter(|(_, d)| d.online)
            .map(|(id, _)| id.clone())
            .collect();
        self.log("Info", format!("Scan complete: {} station(s)", ids.len()));
        for id in ids {
            self.run(&id, Action::Refresh).await;
        }
        Ok(())
    }
    async fn characteristic(&self, p: &Peripheral, uuid: uuid::Uuid) -> Result<Characteristic> {
        for attempt in 0..=2 {
            if let Some(c) = p
                .characteristics()
                .into_iter()
                .find(|c| c.service_uuid == protocol::SERVICE && c.uuid == uuid)
            {
                return Ok(c);
            }
            if attempt == 2 {
                break;
            }
            self.log(
                "Warning",
                format!(
                    "{}: missing {uuid}; discovery attempt {}",
                    p.id(),
                    attempt + 1
                ),
            );
            bounded(p.discover_services()).await?;
            tokio::time::sleep(Duration::from_millis(250)).await;
        }
        bail!(
            "Characteristic {uuid} not found in service {}",
            protocol::SERVICE
        )
    }
    async fn read(&self, p: &Peripheral, c: &Characteristic) -> Result<Vec<u8>> {
        if !c.properties.contains(CharPropFlags::READ) {
            bail!("{} does not support Read", c.uuid);
        }
        let bytes = bounded(p.read(c)).await?;
        self.log(
            "Debug",
            format!("{} RX {} [{}]", p.id(), c.uuid, diagnostics::hex(&bytes)),
        );
        Ok(bytes)
    }
    async fn run(&mut self, id: &str, action: Action) {
        if let Some(d) = self.devices.get_mut(id) {
            d.busy = true;
            d.error = None;
        }
        self.publish(id);
        let result = self.perform(id, action).await;
        if let Err(e) = result {
            self.log("Error", format!("{id} {action:?}: {e:#}"));
            if let Some(d) = self.devices.get_mut(id) {
                d.error = Some(format!("{e:#}"));
                if !action.writes() {
                    d.status = Status::Unavailable;
                    d.channel = None;
                }
            }
        }
        if let Some(p) = self.peripherals.get(id) {
            let connected = bounded(p.is_connected()).await.unwrap_or(false);
            if let Some(d) = self.devices.get_mut(id) {
                d.connected = connected;
                d.online = connected;
            }
        }
        if let Some(d) = self.devices.get_mut(id) {
            d.busy = false;
        }
        self.publish(id);
    }
    async fn perform(&mut self, id: &str, action: Action) -> Result<()> {
        let p = self
            .peripherals
            .get(id)
            .cloned()
            .context("Device unavailable; scan first / Gerät nicht erreichbar")?;
        if !bounded(p.is_connected()).await? {
            self.log("Info", format!("{id}: connecting"));
            let mut connected = false;
            for attempt in 0..2 {
                match bounded(p.connect()).await {
                    Ok(()) => {
                        connected = true;
                        break;
                    }
                    Err(e) => {
                        self.log(
                            "Warning",
                            format!("{id}: connect attempt {}: {e}", attempt + 1),
                        );
                        tokio::time::sleep(Duration::from_millis(500)).await;
                    }
                }
            }
            if !connected {
                bail!("Connection failed after two attempts");
            }
            bounded(p.discover_services()).await?;
            self.log("Info", format!("{id}: GATT discovery complete"));
        }
        if matches!(action, Action::Diagnose) {
            bounded(p.discover_services()).await?;
            let mut report = String::new();
            for service in p.services() {
                report.push_str(&format!("Service {}\n", service.uuid));
                for c in service.characteristics {
                    report.push_str(&format!(
                        "  {} {} {:?}\n",
                        c.uuid,
                        diagnostics::label(c.uuid),
                        c.properties
                    ));
                    if c.properties.contains(CharPropFlags::READ) {
                        match self.read(&p, &c).await {
                            Ok(bytes) => {
                                report
                                    .push_str(&format!("    HEX: {}\n", diagnostics::hex(&bytes)));
                                if let Some(t) = diagnostics::text(&bytes) {
                                    report.push_str(&format!("    TEXT: {t}\n"));
                                }
                            }
                            Err(e) => report.push_str(&format!("    READ ERROR: {e}\n")),
                        }
                    }
                }
            }
            self.devices.get_mut(id).unwrap().diagnosis = report;
        }
        let power = self.characteristic(&p, protocol::POWER).await?;
        let channel = self.characteristic(&p, protocol::CHANNEL).await?;
        // The observed LHB family supports this profile; other generations are read-only.
        let supported = self.devices[id].known.name.starts_with("LHB-");
        self.devices.get_mut(id).unwrap().supported = supported;
        if action.writes() && !supported {
            bail!("Unverified Lighthouse generation: control disabled");
        }
        match action {
            Action::Power(cmd) => {
                if !power
                    .properties
                    .intersects(CharPropFlags::WRITE_WITHOUT_RESPONSE | CharPropFlags::WRITE)
                {
                    bail!("Power characteristic is not writable");
                }
                if !power
                    .properties
                    .contains(CharPropFlags::WRITE_WITHOUT_RESPONSE)
                {
                    self.log("Warning",format!("{id}: WriteWithoutResponse not advertised; using source-confirmed legacy command mode"));
                }
                self.devices.get_mut(id).unwrap().status = Status::Unavailable;
                self.publish(id);
                let io = GattTransport {
                    backend: self,
                    peripheral: &p,
                    power: &power,
                    channel: &channel,
                };
                let outcome = crate::control::power(&io, cmd, Duration::from_secs(1)).await?;
                self.devices.get_mut(id).unwrap().status = outcome.observed;
                self.publish(id);
                if !outcome.confirmed {
                    bail!("Command sent, but requested state not confirmed");
                }
                self.log(
                    "Info",
                    format!(
                        "{id}: power readback matches {cmd:?}; physical readiness not verified"
                    ),
                );
            }
            Action::Channel(value) => {
                if !channel.properties.contains(CharPropFlags::WRITE) {
                    bail!("Channel does not support WriteWithResponse");
                }
                self.devices.get_mut(id).unwrap().channel = None;
                let io = GattTransport {
                    backend: self,
                    peripheral: &p,
                    power: &power,
                    channel: &channel,
                };
                let actual = crate::control::channel(&io, value, Duration::from_millis(500))
                    .await?
                    .byte();
                self.devices.get_mut(id).unwrap().channel = Some(actual);
                if actual != value.byte() {
                    bail!(
                        "Channel mismatch: read {actual}, requested {}",
                        value.byte()
                    );
                }
            }
            _ => {}
        }
        let status = Status::parse(&self.read(&p, &power).await?);
        let actual_channel = Channel::parse(&self.read(&p, &channel).await?);
        let d = self.devices.get_mut(id).unwrap();
        d.status = status?;
        d.channel = Some(actual_channel?.byte());
        d.connected = true;
        d.online = true;
        self.log(
            "Info",
            format!(
                "{id}: status {}, channel {:?}",
                self.devices[id].status, self.devices[id].channel
            ),
        );
        Ok(())
    }
}

struct GattTransport<'a> {
    backend: &'a Backend,
    peripheral: &'a Peripheral,
    power: &'a Characteristic,
    channel: &'a Characteristic,
}
impl crate::control::Transport for GattTransport<'_> {
    async fn write(&self, uuid: uuid::Uuid, byte: u8, acknowledged: bool) -> Result<()> {
        let characteristic = if uuid == protocol::POWER {
            self.power
        } else if uuid == protocol::CHANNEL {
            self.channel
        } else {
            bail!("Unsupported control UUID");
        };
        let mode = if acknowledged {
            WriteType::WithResponse
        } else {
            WriteType::WithoutResponse
        };
        bounded(self.peripheral.write(characteristic, &[byte], mode)).await?;
        self.backend.log(
            "Info",
            format!(
                "{} TX {} [{byte:02X}] {mode:?}; sent, device state not yet confirmed",
                self.peripheral.id(),
                uuid
            ),
        );
        Ok(())
    }
    async fn read(&self, uuid: uuid::Uuid) -> Result<Vec<u8>> {
        let characteristic = if uuid == protocol::POWER {
            self.power
        } else if uuid == protocol::CHANNEL {
            self.channel
        } else {
            bail!("Unsupported control UUID");
        };
        self.backend.read(self.peripheral, characteristic).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn missing_peripherals_are_reported_individually() {
        let (events, rx) = std::sync::mpsc::channel();
        let mut backend = Backend {
            adapter: None,
            peripherals: BTreeMap::new(),
            devices: BTreeMap::new(),
            events,
        };
        for id in ["SIMULATED-a", "SIMULATED-b"] {
            backend.devices.insert(
                id.into(),
                Device {
                    known: KnownDevice {
                        id: id.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                },
            );
            backend.run(id, Action::Refresh).await;
        }
        let errors = rx
            .try_iter()
            .filter(|e| matches!(e, Event::Log("Error", _)))
            .count();
        assert_eq!(errors, 2);
        assert!(
            backend
                .devices
                .values()
                .all(|d| d.error.is_some() && !d.busy)
        );
    }
}
