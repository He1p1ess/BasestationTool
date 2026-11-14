# Basestation Tool (Rust)

Control Valve and HTC Lighthouse base stations individually or as a group over Bluetooth Low Energy (BLE). Turn stations on, switch them to standby or put them to sleep. You can also wake known stations when SteamVR starts or change a station's channel without starting SteamVR.

Basestation Tool is a native Rust desktop application built with egui. Windows 11 and Linux share the same application core. The Linux Bluetooth backend uses btleplug and BlueZ. Most of the project code was generated with AI.

## Installation (v0.1.1)

[Download the Windows and Linux packages from GitHub Releases](https://github.com/He1p1ess/BasestationTool/releases/latest). Each release package has a SHA-256 checksum listed in `SHA256SUMS.txt`.

| Platform | Package | How to launch |
| --- | --- | --- |
| Windows x64 | MSI installer | Double-click `Basestation-Tool-0.1.1-x64.msi`. It installs for the current user. Open **Basestation Tool** from the Start menu. The installer is unsigned. |
| Windows x64 | Portable EXE | Save `Basestation-Tool-0.1.1-x64.exe` in a permanent folder and double-click it. If you move the executable or switch to the installed version, register SteamVR autostart again. |
| Debian or Ubuntu amd64 | DEB | In the download folder, run `sudo apt install ./basestation-tool_0.1.1_amd64.deb`. Launch it from the application menu or run `basestation-tool`. |
| Linux x86-64 | AppImage | Save the AppImage in a permanent folder. Run `chmod +x Basestation-Tool-0.1.1-x86_64.AppImage`, then `./Basestation-Tool-0.1.1-x86_64.AppImage`. If FUSE is unavailable, run `./Basestation-Tool-0.1.1-x86_64.AppImage --appimage-extract-and-run`. |

You need a Bluetooth LE adapter with working drivers. Linux also requires a graphical session, glibc 2.31 or later, graphics libraries, D-Bus and BlueZ.

## First launch

Connect your stations to power, enable Bluetooth and open the app. By default, it scans for nearby stations and reads their status and channel. A normal launch only reads device data. Manual power and channel changes show a confirmation dialog listing the target stations. Automatic SteamVR power actions are off by default.

Only one instance runs per user. A second instance exits immediately.

## Using the app

- **Find stations.** A scan lasts eight seconds and finishes with a status check. Automatic scanning runs every 60 seconds and can be turned off in Settings. While idle, the app checks status every 30 seconds.
- **Select and control stations.** Click a station name to select it alone, or use the checkboxes to select a group. The highlighted rows and the names above the action bar show which stations will receive **On**, **Standby**, **Sleep** or **Refresh status**. The app captures the selection when you click the action.
- **Identify a station.** Select exactly one station and choose **Identify** to blink its LED. If it was in sleep or standby, the tool restores that state after 15 seconds and checks the readback.
- **Manage one station.** Select exactly one station to edit its local display name, set its channel or open diagnostics. Renaming affects only the name shown in the app. On a supported LHB station, choose a channel from 1 to 16 and confirm. The app reads the channel back from the station and displays that value.
- **Read diagnostics.** Inspect GATT services, characteristics, properties, hex values and validated text, including the manufacturer, model, serial number and firmware. You can refresh, select and copy text, or export the full report.
- **Read signal strength.** RSSI shows the last signal strength observed during a scan, in dBm. Hover for the observation time. A dash means that no reading is available. RSSI is not a distance measurement.
- **Arrange the table and log.** Click a column heading to sort and click again to reverse the order. Channel and RSSI sort numerically. Selections follow device IDs when rows move. Drag column boundaries to resize them, or scroll horizontally in a narrow window. With the log visible, drag the divider beneath the table to resize both areas. Increasing the window height gives the log more room.
- **Inspect and export logs.** Search the live log, filter by level, clear it or save it. BLE debugging includes byte values. Technical log messages and GATT data use English in every interface language. The app keeps up to 2,000 entries in memory and rotates the current log plus four older files at about 2 MB each. Exporting a log or diagnostic report opens the destination folder and displays the full file path.
- **Reset Bluetooth.** This closes the app's connections and reinitializes scanning. The system adapter stays enabled and Windows settings are preserved.
- **Change preferences.** Open the gear in the upper-right corner for **Language**, **Show log**, appearance (system, dark or light), scanning, logging and SteamVR options. The interface supports German, English, French, Spanish and Italian. eframe remembers the window size and position.

## SteamVR integration

1. Place the executable in its permanent location and start SteamVR.
2. In **Settings**, enable **Automatically start with SteamVR**. The app saves this preference once OpenVR confirms registration. If registration fails, it displays the OpenVR error.
3. To wake known stations as well, enable **Power on when SteamVR starts**. Under **When SteamVR exits**, choose **No action**, **Standby** or **Sleep**. These settings authorize power commands to all known stations.
4. To disable autostart, clear **Automatically start with SteamVR** while SteamVR is running.

**Close with SteamVR** applies to instances SteamVR launched through the manifest with `--steamvr`. An instance you opened manually stays open. The app waits for queued power actions before closing. A separate monitor watches the `vrserver` process without starting SteamVR itself.

Registration uses an OpenVR application manifest and the calls `AddApplicationManifest`, `SetApplicationAutoLaunch` and `GetApplicationAutoLaunch`. This requires no registry startup entries or system-wide scheduled tasks. Although the manifest uses overlay registration, the app displays a desktop window.

The app locates the installed OpenVR library through `openvrpaths.vrpath`. You can also provide an absolute library path in Settings. The usual Windows location within the Steam library is `SteamVR/bin/win64/openvr_api.dll`. The app uses the installed library without downloading a DLL.

## Files and settings

The app uses `directories::ProjectDirs` to locate the current user's configuration directory. Settings shows the exact path.

- Windows uses `%APPDATA%\BasestationTool\BasestationTool\config`.
- Linux typically uses `~/.config/basestationtool`.

`settings.json` stores known device IDs, local names and preferences. Before replacing it, the app creates `settings.previous.json`. If the JSON is invalid, the app displays an error and disables saving until you repair the file outside the app. Known stations appear offline at startup until the app gets a fresh status reading.

The app writes logs to `logs/` and exports to `exports/` inside the user data directory. Logs and diagnostic reports can contain device addresses and serial numbers. The project's original reference files are kept separately.

For isolated tests, set `BASESTATION_DATA_DIR` to a different data directory. Launching with `--no-ble` disables scanning, reading and writing and marks the window accordingly. Simulated devices are available only through the unit-test transport.

## Building on Windows

Use stable Rust, the x64 MSVC toolchain, Visual Studio Build Tools with the **Desktop development with C++** workload, and the Windows SDK. The documented build used Rust and Cargo 1.99.1. Cargo needs access to the dependencies pinned in `Cargo.lock`.

```powershell
cargo fmt --check
cargo check --locked
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo build --locked --release
.\target\release\basestation-tool.exe
```

If Cargo is missing from `PATH`, use `%USERPROFILE%\.cargo\bin\cargo.exe`. Running the finished executable requires the usual Windows and graphics drivers and, where applicable, the MSVC runtime. Both Windows downloads run without Cargo or .NET installed.

## Building on Linux

The distributed x86-64 packages were cross-compiled on Windows for GNU/Linux with a glibc 2.31 target. To build natively on Debian or Ubuntu, install Rust and the system dependencies, then build and launch the app.

```sh
sudo apt install build-essential pkg-config libdbus-1-dev libudev-dev libx11-dev libxi-dev libxrandr-dev libxcursor-dev libxinerama-dev libxkbcommon-dev libwayland-dev libegl1-mesa-dev bluez
cargo build --locked --release
./target/release/basestation-tool
```

Run the GUI as your regular desktop user. BlueZ and the system D-Bus must be running, and your distribution's policy must allow Bluetooth access for that user. The app reports backend errors for missing permissions or adapter failures. SteamVR integration requires a Linux runtime containing `bin/linux64/libopenvr_api.so`.

## Architecture and protocol

| Module | Responsibility |
| --- | --- |
| `gui` | Interface, target selection and confirmation dialogs |
| `device` | Stable device identities and target sets |
| `ble` | Adapter access, discovery, bounded command queue and timeouts |
| `protocol` | UUIDs, typed commands, status and channel validation |
| `control` | Independently testable write and readback operations |
| `diagnostics` | Hex and text validation, standard device information |
| `config` | Persistent user preferences |
| `logging` | Bounded live log, file rotation and exports |
| `steamvr` | OpenVR ABI and process monitoring |

The power service UUID is `00001523-1212-efde-1523-785feabcd124`. The app writes a single byte to the power characteristic, `00001525-1212-efde-1523-785feabcd124`, using a GATT write without response.

| Command | Byte (hex) |
| --- | --- |
| On | `01` |
| Standby | `02` |
| Sleep | `00` |

The app waits one second before reading back the state and makes up to three read attempts. The station can report `01`, `09` or `0B` as awake. The display includes the raw byte. A reported awake state alone cannot confirm that the motor and laser are ready. On one tested station waking from sleep, the readback progressed through `08`, `01` and `09` before settling at `0B` roughly 11 seconds after the command. During a wake from a known sleep or standby state, the app labels `01` and `09` as **Starting up** and briefly refreshes the station more frequently. Checking the original executable's button handlers confirmed that the wake command is `01`. The `0B` byte observed in diagnostics was a readback value. Unrecognized status bytes appear as unknown.

For channel changes, the app writes one byte (1–16) to `00001524-1212-efde-1523-785feabcd124` using a GATT write with response, then reads the channel back. The app never automatically repeats a write.

Writing is enabled only for the recognized LHB profile. Device identity combines the operating-system prefix with the backend's `PeripheralId`. A service UUID or local name cannot identify an individual station. Other hardware generations and rotating BLE privacy addresses may require separate support. The app keeps changed identities separate.

## Testing and known limitations

The Windows release executable was tested with three physical LHB stations. The project includes a [protocol analysis](docs/ANALYSE.md) and [detailed test results](docs/TESTS.md), both in German.

- **Power and channels.** Hardware tests exercised single-station, selected-group and all-station commands and verified channel readback on channels 1, 5 and 16. An early power-on test returned `0B` while the station remained inactive. The wake command was corrected to `01`. The owner subsequently reported that power-on worked and had also confirmed sleep and standby.
- **Adapter and device support.** All three tested stations accepted power writes without response even though they did not advertise the `WriteWithoutResponse` property. Other device and driver combinations still need testing. The app selects one adapter automatically. Manual adapter selection and per-column hiding are currently unavailable.
- **SteamVR.** On October 20, 2026, registration was enabled, verified and disabled again with SteamVR 2.17.10. A subsequent SteamVR restart returned `HmdNotFound` (108) because no headset was connected. End-to-end testing of automatic launch, shutdown and power actions still requires a working VR session.
- **Linux.** The Debian package, its ELF executable and the AppImage were inspected. The Linux GUI, Bluetooth operations and SteamVR integration still need testing on a Linux desktop.

Technical references include the [btleplug API](https://docs.rs/btleplug/0.13.5/btleplug/api/trait.Peripheral.html), the [eframe API](https://docs.rs/eframe/0.37.0/eframe/) and [Valve's OpenVR header](https://github.com/ValveSoftware/openvr/blob/master/headers/openvr_capi.h).
