use crate::protocol::{Channel, PowerCommand, Status};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct KnownDevice {
    pub id: String,
    pub name: String,
    pub alias: String,
    pub address: String,
}
#[derive(Debug, Clone, Default)]
pub struct Device {
    pub known: KnownDevice,
    pub status: Status,
    pub channel: Option<u8>,
    pub rssi: Option<i16>,
    pub seen: Option<String>,
    pub online: bool,
    pub connected: bool,
    pub supported: bool,
    pub busy: bool,
    pub error: Option<String>,
    pub diagnosis: String,
}
impl Device {
    pub fn display_name(&self) -> &str {
        if self.known.alias.trim().is_empty() {
            &self.known.name
        } else {
            &self.known.alias
        }
    }
}
#[derive(Debug, Clone, Copy)]
pub enum Action {
    Refresh,
    Diagnose,
    Power(PowerCommand),
    Channel(Channel),
}
impl Action {
    pub fn writes(self) -> bool {
        matches!(self, Self::Power(_) | Self::Channel(_))
    }
}
pub enum Scope<'a> {
    One(&'a str),
    Selected(&'a BTreeSet<String>),
    All,
}
pub fn targets(devices: &BTreeMap<String, Device>, scope: Scope<'_>) -> Vec<String> {
    devices
        .iter()
        .filter(|(id, _)| match &scope {
            Scope::One(one) => id.as_str() == *one,
            Scope::Selected(set) => set.contains(*id),
            Scope::All => true,
        })
        .map(|(id, _)| id.clone())
        .collect()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn identities_and_scopes() {
        let mut devices = BTreeMap::new();
        for id in ["a", "b", "c"] {
            devices.insert(
                id.into(),
                Device {
                    known: KnownDevice {
                        id: id.into(),
                        alias: "Same name".into(),
                        ..Default::default()
                    },
                    ..Default::default()
                },
            );
        }
        assert_eq!(targets(&devices, Scope::One("b")), vec!["b"]);
        assert_eq!(
            targets(
                &devices,
                Scope::Selected(&BTreeSet::from(["a".into(), "c".into()]))
            ),
            vec!["a", "c"]
        );
        assert_eq!(targets(&devices, Scope::All).len(), 3);
        devices.get_mut("b").unwrap().known.alias = "Changed".into();
        assert_eq!(targets(&devices, Scope::One("b")), vec!["b"]);
    }
}
