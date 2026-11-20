# Test report

These results cover the Windows x64 builds tested on October 19–20, 2026, with Rust and Cargo 1.99.1. The test team did not run the legacy executable. On October 20, the SHA-256 hashes of all ten reference files matched the original inventory. Device names and addresses in the published report are anonymized.

## Automated checks

| Check | Result for the initial build |
| --- | --- |
| `cargo fmt --check` | Passed |
| `cargo check --locked` | Passed |
| `cargo clippy --locked --all-targets --features hardware-tests -- -D warnings` | Passed without warnings. The default build had also been checked. |
| `cargo test --locked` | 13 passed, 0 failed |
| `cargo build --release` | Produced the native Windows executable at `target/release/basestation-tool.exe` (approximately 8.9 MB). |

Coverage included the specified power commands and status bytes, unknown values, invalid response lengths, channels 1, 5 and 16, and out-of-range channels. Device tests covered stable identities despite identical or editable display names, individual and group target selection, name and settings serialization, disabled automation by default, bounded log storage, and separation of binary data from readable text.

Transaction tests checked power writes without response, channel writes with response, delayed readback confirmation, successful writes without a matching readback, and display of the channel actually read from the device. Transport tests used `SimulatedTransport`. The backend error-isolation test used missing simulated IDs to check per-device failures without opening an adapter.

## Initial GUI checks

The Windows release executable was launched with `--no-ble` and an isolated data directory under `target/ui-test-data`.

- The main window and Settings were visible and usable. Dark and light modes were checked visually and the contrast of a warning message was corrected.
- The interface switched between German and English. The selected language and theme were restored from the test configuration after restarting.
- Three local rows labeled `SIMULATED-*` loaded offline with no fabricated RSSI or status values.
- The checkbox for test device A and individual focus for device B could be selected independently. Both target counters were correct.
- Closing the window exited the app successfully.

The checks used a separate configuration and offline rows. The GUI has no simulated live mode.

## BLE reads on physical stations

The release executable was started normally with a separate configuration under `target/hardware-read-test`. Automatic power actions were disabled. This run only scanned, connected, discovered services and read characteristics.

| Device | Address | Status read | Channel read |
| --- | --- | --- | --- |
| Station 1 | Redacted | Sleep / 00 | 5 / 05 |
| Station 2 | Redacted | Sleep / 00 | 9 / 09 |
| Station 3 | Redacted | Sleep / 00 | 1 / 01 |

The first scan ran from 22:09:13 to 22:09:21. All status reads finished by 22:09:29. A second status and channel read succeeded at 22:09:43. These observations verified Windows BLE detection, GATT discovery and readback on the three devices.

Between 22:11:20 and 22:11:22, diagnostics read Station 1's readable GATT characteristics, including model, serial number, firmware and manufacturer. The diagnostic report was exported through the GUI and checked on disk. The second scan displayed actual RSSI readings of −58, −66 and −52 dBm.

The live test exposed a shared scroll ID between the table and log. After assigning distinct IDs, the device table was checked visually again. Formatting, compiler checks, Clippy, unit tests and the release build passed after the fix.

Local evidence includes `hardware-read-test.log` and `hardware-diagnosis.txt`. The log contains no `TX` entries. The test instance was closed with SteamVR registration untouched.

## Authorized hardware acceptance test, October 20, 2026

With the owner's approval, the hardware runner used the application's BLE backend, transport and target-selection code to send commands to physical devices. The runner requires the `hardware-tests` Cargo feature and the `--hardware-acceptance` argument. It is excluded from the standard release executable.

- The runner issued On, Standby and Sleep to one station, a selection of two stations and all three stations. All nine combinations met the readback checks.
- After every step, it read status and channel from all stations. Stations outside the target set retained their values.
- Power writes without response succeeded even though the devices did not advertise the WriteWithoutResponse property.
- Station 1 accepted channels 1, 5 and 16 using writes with response. Each channel was read back correctly.
- The build under test preserved an intermediate power byte of `09` as unknown.
- The runner saved the baseline before the first write. At 15:17:34 it restored and verified the following values.

| Station | Restored state | Channel |
| --- | --- | --- |
| Station 1 | On | 5 |
| Station 2 | Sleep | 9 |
| Station 3 | Sleep | 1 |

The local files `hardware-acceptance-20261020.log` and `hardware-baseline-20261020.json` record the result `ACCEPTANCE RESULT: Ok(())`. This run verified BLE responses. It included no independent observation of the motor or laser.

## Wake command correction and follow-up, October 20, 2026

After the acceptance run, the owner reported that a station could return `0B` without turning green or becoming ready. The earlier claim that the wake hardware test had passed overstated what readback alone established. Rechecking the original executable's IL showed that it sent `01` to wake a station, while the Rust app had sent `0B`. See the [wake command investigation](ANALYSE.md#wake-command-investigation-october-20-2026).

A regression test verified that wake always writes `01` without a GATT response, preserves the reported bytes `01`, `09` and `0B`, and leaves `0C` unknown. After the correction, **14 tests passed**, Clippy passed without warnings for all targets with `hardware-tests` enabled, and the release build succeeded. This software check sent no power or channel commands. The hardware logs above describe the earlier build. The owner subsequently confirmed that the corrected version physically woke the station.

## GUI follow-up, October 20, 2026

After the owner confirmed wake-up, the interface gained a language dropdown, a readable selection heading, a single-device area with name and address, buttons of consistent height, and a Settings gear in the upper-right corner. The release build succeeded and Clippy found no warnings on the default targets.

An isolated `--no-ble` instance under `target/ui-layout-test` was used to check device focus, the German/English dropdown, single-device controls and Settings. Exporting a log opened Explorer at the correct test export directory. The run used the isolated configuration and sent no hardware commands.

A further layout check covered the language and persistent log-visibility controls in Settings, diagnostics below the channel controls, sortable column headings with reversible order, and selection by device ID. In the isolated instance, name sorting was verified in both directions. Dragging the divider's middle handle resized the table and log. Maximizing the window preserved the chosen table height and gave the additional space to the log, which had no fixed upper height limit. Clippy and the release build passed.

## Authorized SteamVR test, October 20, 2026

SteamVR 2.17.10 was started. On the first attempt, error 108 exposed an incorrect OpenVR initialization type. The code was changed from Background (3) to Utility (4), which avoids headset and driver initialization.

Autostart was then enabled through the GUI. `AddApplicationManifest`, `SetApplicationAutoLaunch` and `GetApplicationAutoLaunch` succeeded. The setting persisted after restarting the tool. After closing both applications and restarting SteamVR, runtime startup failed with `VRInitError_Init_HmdNotFound` (108). The tool did not autostart during that attempt. Registration was verified, but end-to-end autostart still needed a working VR session.

Autostart was disabled again through the GUI and checked through OpenVR. SteamVR's `org.basestationtool.desktop.vrappconfig` recorded `autolaunch: false` and `last_launch_time: "0"`. Automatic power actions and the option to close with SteamVR stayed disabled. Both programs were closed. The last observed station states and channels were On/5, Sleep/9 and Sleep/1.

Local evidence includes `steamvr-runtime-test-20261020.log` (relevant runtime excerpts) and `steamvr-restored-20261020.json` (autostart disabled).

## Practical checks still open at the end of the October test run

- SteamVR autostart with a connected headset, single-instance handling on this launch path and optional automatic shutdown.
- Automatic SteamVR power actions after explicit opt-in.
- Physical signal or power loss, a disabled adapter, missing services and real timeouts. Simulated tests covered per-device error isolation.
- Persistence of a real device's local alias across restarts. Automated tests covered serialization and stable identities.
- Linux builds and runtime behavior, and other Lighthouse generations. Linux packages were built and verified in later releases. Linux desktop operation remains a separate test.

## Release 0.1.2 (2026-11-20)

Formatting, all 15 unit tests and Clippy (all targets, hardware-tests feature) passed. Windows and Linux release builds succeeded. WiX validated the per-user MSI (expected ICE91 warning). The DEB binary was compared with the Linux build, and the AppImage packer verified its extracted payload and links. All five website languages were checked for current download URLs and retained changelog history. Linux desktop/Bluetooth operation was not tested for this release. No hardware commands were sent during packaging.
