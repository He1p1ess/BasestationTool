use crate::config;
use std::{collections::VecDeque, io::Write};
#[derive(Clone)]
pub struct Entry {
    pub level: &'static str,
    pub line: String,
}
#[derive(Default)]
pub struct Logs {
    pub entries: VecDeque<Entry>,
    pub disk_error: Option<String>,
}
impl Logs {
    pub fn push(&mut self, level: &'static str, message: impl AsRef<str>, disk: bool) {
        let line = format!(
            "{} [{level}] {}",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
            message.as_ref()
        );
        if disk && let Err(e) = append(&line) {
            self.disk_error = Some(e.to_string());
        }
        self.entries.push_back(Entry { level, line });
        while self.entries.len() > 2000 {
            self.entries.pop_front();
        }
    }
    pub fn full_text(&self) -> String {
        self.entries
            .iter()
            .map(|e| e.line.as_str())
            .collect::<Vec<_>>()
            .join("\n")
    }
}
fn append(line: &str) -> std::io::Result<()> {
    let dir = config::directory().join("logs");
    std::fs::create_dir_all(&dir)?;
    let path = dir.join("current.log");
    if std::fs::metadata(&path).is_ok_and(|m| m.len() > 2_000_000) {
        for i in (1..4).rev() {
            let old = dir.join(format!("previous-{i}.log"));
            if old.exists() {
                std::fs::rename(old, dir.join(format!("previous-{}.log", i + 1)))?;
            }
        }
        std::fs::rename(&path, dir.join("previous-1.log"))?;
    }
    writeln!(
        std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?,
        "{line}"
    )
}
pub fn export(prefix: &str, text: &str) -> std::io::Result<std::path::PathBuf> {
    let dir = config::directory().join("exports");
    std::fs::create_dir_all(&dir)?;
    let path = dir.join(format!(
        "{prefix}-{}.txt",
        chrono::Local::now().format("%Y%m%d-%H%M%S-%f")
    ));
    std::fs::write(&path, text)?;
    Ok(path)
}

pub fn open_export_folder(path: &std::path::Path) -> std::io::Result<()> {
    let folder = path.parent().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Export has no parent folder",
        )
    })?;
    #[cfg(target_os = "windows")]
    let mut command = std::process::Command::new("explorer.exe");
    #[cfg(not(target_os = "windows"))]
    let mut command = std::process::Command::new("xdg-open");
    command.arg(folder).spawn()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn bounded_memory_retains_latest_entries() {
        let mut log = super::Logs::default();
        for i in 0..2100 {
            log.push("Info", format!("Entry {i}"), false);
        }
        assert_eq!(log.entries.len(), 2000);
        assert!(log.entries.back().unwrap().line.ends_with("Entry 2099"));
    }
}
