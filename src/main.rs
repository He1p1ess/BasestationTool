#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
mod ble;
mod config;
mod control;
mod device;
mod diagnostics;
mod gui;
mod i18n;
mod logging;
mod protocol;
mod steamvr;

fn main() -> eframe::Result<()> {
    let dir = config::directory();
    if let Err(e) = std::fs::create_dir_all(&dir) {
        eprintln!("Cannot create data directory: {e}");
        return Ok(());
    }
    let lock = std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(dir.join("instance.lock"));
    let Ok(lock) = lock else {
        return Ok(());
    };
    if fs2::FileExt::try_lock_exclusive(&lock).is_err() {
        return Ok(());
    }
    #[cfg(feature = "hardware-tests")]
    if std::env::args().any(|arg| arg == "--hardware-acceptance") {
        let result = ble::acceptance::run();
        if let Err(error) = result {
            eprintln!("{error:#}");
            std::process::exit(1);
        }
        return Ok(());
    }
    let no_ble = std::env::args().any(|a| a == "--no-ble");
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1100.0, 760.0])
            .with_min_inner_size([720.0, 580.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Basestation Tool",
        options,
        Box::new(move |_| Ok(Box::new(gui::App::new(no_ble)))),
    )
}
