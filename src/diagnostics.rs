pub fn hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| format!("{b:02X}"))
        .collect::<Vec<_>>()
        .join(" ")
}
pub fn text(bytes: &[u8]) -> Option<&str> {
    std::str::from_utf8(bytes).ok().filter(|s| {
        !s.trim().is_empty()
            && s.chars()
                .all(|c| !c.is_control() || c == '\n' || c == '\r' || c == '\t')
    })
}
pub fn label(uuid: uuid::Uuid) -> &'static str {
    match uuid.as_u128() {
        0x00002a29_0000_1000_8000_00805f9b34fb => "Manufacturer",
        0x00002a24_0000_1000_8000_00805f9b34fb => "Model",
        0x00002a25_0000_1000_8000_00805f9b34fb => "Serial",
        0x00002a26_0000_1000_8000_00805f9b34fb => "Firmware",
        _ => "",
    }
}
#[cfg(test)]
mod tests {
    #[test]
    fn binary_is_not_text() {
        assert!(super::text(&[0, 1, 2]).is_none());
        assert!(super::text(&[255]).is_none());
        assert!(super::text(&[9]).is_none());
        assert_eq!(super::text(b"Valve Corp."), Some("Valve Corp."));
    }
}
