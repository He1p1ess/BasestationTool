//! OpenVR ABI boundary. Registration is only called following an explicit settings change.
use anyhow::{Context, Result, bail};
use std::{
    ffi::{CString, c_char, c_void},
    path::{Path, PathBuf},
};
const KEY: &str = "org.basestationtool.desktop";
const VR_APPLICATION_UTILITY: i32 = 4;
#[repr(C)]
struct Applications {
    add: unsafe extern "system" fn(*const c_char, bool) -> i32,
    _slots_1_to_16: [*const c_void; 16],
    set_auto: unsafe extern "system" fn(*const c_char, bool) -> i32,
    get_auto: unsafe extern "system" fn(*const c_char) -> bool,
}
pub fn library_path(custom: &str) -> Result<PathBuf> {
    if !custom.trim().is_empty() {
        let path = PathBuf::from(custom);
        if !path.is_absolute() {
            bail!("OpenVR library path must be absolute");
        }
        return Ok(path);
    }
    let config = if cfg!(windows) {
        PathBuf::from(std::env::var_os("LOCALAPPDATA").context("LOCALAPPDATA missing")?)
            .join("openvr/openvrpaths.vrpath")
    } else {
        directories::BaseDirs::new()
            .context("Home missing")?
            .config_dir()
            .join("openvr/openvrpaths.vrpath")
    };
    let value: serde_json::Value = serde_json::from_slice(
        &std::fs::read(config).context("SteamVR is not installed / SteamVR nicht installiert")?,
    )?;
    let runtime = value["runtime"]
        .as_array()
        .and_then(|v| v.first())
        .and_then(|v| v.as_str())
        .context("OpenVR runtime path missing")?;
    let path = Path::new(runtime).join(if cfg!(windows) {
        "bin/win64/openvr_api.dll"
    } else {
        "bin/linux64/libopenvr_api.so"
    });
    if !path.is_file() {
        bail!(
            "OpenVR library not found: {}. Select the official runtime library in settings.",
            path.display()
        );
    }
    Ok(path)
}
pub fn register(enabled: bool, custom: &str) -> Result<()> {
    let library = library_path(custom)?;
    let executable = std::env::current_exe()?;
    // SteamVR must launch the persistent AppImage, not its temporary mounted ELF.
    #[cfg(target_os = "linux")]
    let executable = match (std::env::var_os("APPDIR"), std::env::var_os("APPIMAGE")) {
        (Some(dir), Some(image))
            if Path::new(&dir).is_absolute()
                && executable.starts_with(&dir)
                && Path::new(&image).is_absolute()
                && Path::new(&image).is_file() =>
        {
            PathBuf::from(image)
        }
        _ => executable,
    };
    let manifest = serde_json::json!({"source":"builtin","applications":[{
        "app_key":KEY,"launch_type":"binary","binary_path_windows":executable,
        "binary_path_linux":executable,"arguments":"--steamvr","is_dashboard_overlay":true,
        "strings":{"en_us":{"name":"Basestation Tool","description":"Lighthouse BLE controller"}}
    }]});
    let dir = crate::config::directory();
    std::fs::create_dir_all(&dir)?;
    let path = dir.join("basestation.vrmanifest");
    std::fs::write(&path, serde_json::to_vec_pretty(&manifest)?)?;
    // SAFETY: Symbols and table layout match Valve's openvr_capi.h IVRApplications_008.
    // The library stays loaded until all calls and VR_ShutdownInternal have completed.
    unsafe {
        let lib = libloading::Library::new(library)?;
        let init = lib.get::<unsafe extern "C" fn(*mut i32, i32) -> u32>(b"VR_InitInternal\0")?;
        let shutdown = lib.get::<unsafe extern "C" fn()>(b"VR_ShutdownInternal\0")?;
        let interface = lib.get::<unsafe extern "C" fn(*const c_char, *mut i32) -> *mut c_void>(
            b"VR_GetGenericInterface\0",
        )?;
        let mut error = 0;
        init(&mut error, VR_APPLICATION_UTILITY); // No driver or HMD initialization.
        if error != 0 {
            bail!("OpenVR initialization error {error}; start SteamVR and retry");
        }
        let result = (|| -> Result<()> {
            let ptr = interface(c"FnTable:IVRApplications_008".as_ptr(), &mut error);
            if error != 0 || ptr.is_null() {
                bail!("OpenVR applications interface error {error}");
            }
            let apps = &*ptr.cast::<Applications>();
            let key = CString::new(KEY)?;
            let manifest = CString::new(path.to_string_lossy().as_bytes())?;
            let error = (apps.add)(manifest.as_ptr(), false);
            if error != 0 {
                bail!("AddApplicationManifest error {error}");
            }
            let error = (apps.set_auto)(key.as_ptr(), enabled);
            if error != 0 {
                bail!("SetApplicationAutoLaunch error {error}");
            }
            if (apps.get_auto)(key.as_ptr()) != enabled {
                bail!("SteamVR autostart verification failed");
            }
            Ok(())
        })();
        shutdown();
        result
    }
}
pub struct Monitor {
    pub rx: std::sync::mpsc::Receiver<bool>,
    stop: std::sync::mpsc::Sender<()>,
}
impl Monitor {
    pub fn start() -> Self {
        let (tx, rx) = std::sync::mpsc::channel();
        let (stop, exit) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let mut system = sysinfo::System::new();
            let mut previous = None;
            loop {
                system.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
                let running = system
                    .processes()
                    .values()
                    .any(|p| matches!(p.name().to_str(), Some("vrserver.exe" | "vrserver")));
                if previous != Some(running) {
                    if tx.send(running).is_err() {
                        break;
                    }
                    previous = Some(running);
                }
                if exit.recv_timeout(std::time::Duration::from_secs(2))
                    != Err(std::sync::mpsc::RecvTimeoutError::Timeout)
                {
                    break;
                }
            }
        });
        Self { rx, stop }
    }
}
impl Drop for Monitor {
    fn drop(&mut self) {
        let _ = self.stop.send(());
    }
}
