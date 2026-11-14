# Protocol analysis and architecture

This report records the investigation that began on October 19, 2026. The reference logs, screenshots and files under `BaseStation Tool Old` were examined read-only. The local SHA-256 inventory is `reference-hashes.json`. The investigation used static analysis of the original executable.

## Reference material

- The two screenshots show a compact WPF table with status queries, scan and reset controls, individual and group commands, channel selection, diagnostics, log export and language selection. The original interface had no separate checkboxes for selecting multiple stations.
- `Logs/log.txt` records three Valve Corp. model 1004 devices. Stations 3, 2 and 1 were on channels 1, 9 and 5 respectively. Their addresses have been anonymized. Diagnostics listed six services per device and recorded power values `00` and `0B`.
- Standard services included `1800`, `1801` and `180A`. The Device Information service (`180A`) exposed manufacturer (`2A29`), model (`2A24`), serial number (`2A25`), hardware revision (`2A27`) and firmware revision (`2A26`). Diagnostics also listed proprietary services and the DFU service (`FE59`). These observations alone provided no basis for sending writes.
- The 238,449,392-byte executable is a .NET 8 single-file WPF bundle for Windows x64. ILSpy 11.1 extracted intermediate language (IL) from the embedded `Basestation Tool.dll`. A metadata resolution error prevented complete C# decompilation of the ReadyToRun variant. The regular assembly's readable IL is stored locally under `tools/analysis`.
- The separate DLLs supplied Microsoft WPF, graphics and Visual C++ runtime components. The bundle also included a PDB with debugging symbols. No additional application source files were found alongside it.

## Protocol evidence

| Finding | Evidence |
| --- | --- |
| Service `1523`, power `1525`, channel `1524` | Owner-supplied code fragments, device logs and IL agree. |
| Power commands `00` (sleep), `02` (standby) and `01` (on), written without response | On October 20, inspection of both wake button handlers showed `ldc.i4.1`. The write routine passed the byte through unchanged with `GattWriteOption=1`. The earlier identification of `0B` as the wake command came from a readback. |
| Channels 1–16, written with response | The supplied code specifies the range and write mode. The channel handler's IL references `1524` and `WriteValueAsync`. |
| Power readback | Around IL lines 3451–3500, UUID `1525` is followed by `ReadValueAsync` with uncached reads (`CacheMode=1`). |
| Channel readback | The IL reads characteristic `1524` around lines 3589–3638. |
| Reported power values | The reference logs contain `00` and `0B`. The standby mapping `02` initially came from the supplied code. |
| Unknown status bytes | The project brief identified a bug in the legacy logic that treated unknown values as on. The Rust parser preserves the unrecognized byte. |
| Power write properties | The original logs advertised Read, Write and Notify. The Rust app logs the missing WriteWithoutResponse flag and uses the write mode found in the original app. Subsequent hardware tests are recorded in the [test report](TESTS.md). |
| Other Lighthouse generations | Only stations with the recognized LHB profile were approved for writes. |

The original diagnostics attempted to display arbitrary binary data as text. The Rust diagnostics always show hex and add text only for valid UTF-8 without disallowed control characters.

## Implementation

1. `protocol.rs` handles protocol values and validation independently of the OS and GUI.
2. `device.rs` manages device identities, local names and target sets. Service UUIDs identify services, not individual stations.
3. `ble.rs` uses btleplug 0.13.5. One worker processes a bounded command queue, one device operation at a time. A per-device error is logged and processing continues with the rest of the group. BLE calls have a 12-second timeout. The backend allows two connection attempts and up to two additional discovery attempts. It never retries a write automatically.
4. The original `gui.rs` implementation used eframe/egui 0.37, resizable table columns, German and English labels, system/dark/light themes and individual focus independent of checkbox selection.
5. Separate modules handle settings, rotating logs and diagnostics. Invalid configuration files are preserved. In `control.rs`, a transport trait separates power and channel transactions from the BLE backend so unit tests can use a simulated transport.
6. `steamvr.rs` registers the application through an OpenVR application manifest and the `IVRApplications_008` calls `AddApplicationManifest`, `SetApplicationAutoLaunch` and `GetApplicationAutoLaunch`. Registration requires a user action. A separate thread monitors the process and a file lock enforces a single application instance.
7. Validation included formatting, compiler checks, Clippy, unit tests, a release build, a GUI launch with BLE disabled and documented hardware tests.

## Caching and device identity

Inspection of btleplug 0.13.5 showed that Windows characteristic reads use uncached values. Service and characteristic discovery also starts uncached, but may use cached characteristic metadata after a timeout. The app reuses discovered characteristics and repeats discovery after reconnecting or when a UUID is missing. The scan processes current advertising events. RSSI shows the value from the last scan that observed the station.

A device ID combines the OS prefix with the backend's `PeripheralId`. It is address-based on Windows and backend-dependent on Linux. Devices with changing addresses are kept separate even if they share a name, so commands stay bound to the selected identity. Persistent identification across privacy address changes remains a hardware and backend limitation.

## Wake command investigation, October 20, 2026

The Rust app reported `0B`, but the owner observed that the station's LED did not turn green. The original C# app could wake the same station. The IL for both original button handlers showed which argument was passed:

```text
<btnModeOn_Click>d__16.MoveNext:
IL_0012: ldc.i4.1
IL_0013: call ... MainWindow::SendPowerCommandAsync(uint8)

<btnModeOnAll_Click>d__19.MoveNext:
IL_0012: ldc.i4.1
IL_0013: call ... MainWindow::SendPowerCommandToAllAsync(uint8)
```

These calls appear around lines 5111 and 4977 respectively in the local `tools/analysis/Basestation Tool.il`. The initial analysis examined the write routine but missed its callers' arguments. The Rust implementation consequently sent the readback value `0B` until the command was corrected to `01`.

Two original diagnostic exports, `Logs/Log-gestartet-über-das-CSharp-Tool.txt` and `Logs/Log-gestartet-über-das-Rust-Tool.txt`, differed at one byte in the 64-byte block from `00000010-0060-7990-5544-1cce81af42f0`. At zero-based offset 20, the Rust capture contained `0B` and the C# capture contained `0C`. Both reads of the power characteristic (`1525`) returned `0B`. The additional characteristic is read-only. Those two snapshots alone could not establish whether the byte was a counter or a state indicator. Diagnostics preserve its raw value.

Independent community implementations also distinguish the `01` wake command from the reported awake values `01`, `09` and `0B`. The corrected October build preserved these raw bytes and displayed “On (reported)”. Other unrecognized bytes remained unknown. These findings come from reverse engineering rather than an official Valve protocol specification. [Ben Woodford's GATT notes](https://gist.github.com/BenWoodford/3a1e500a4ea2673525f5adb4120fd47c) and [lighthouse-v2-manager](https://github.com/nouser2013/lighthouse-v2-manager/blob/master/lighthouse-v2-manager.py).

The Rust app serializes BLE requests and waits one second between a power write and readback. The original IL also contained a 1,500 ms delay on a retry or error path. The investigation found no evidence that Rust's read frequency prevented wake-up. The fix changed the command byte without adding extra writes or retries.

## Library references

- [btleplug Peripheral API, version 0.13.5](https://docs.rs/btleplug/0.13.5/btleplug/api/trait.Peripheral.html)
- [eframe API, version 0.37.0](https://docs.rs/eframe/0.37.0/eframe/)
- [Valve OpenVR C API header](https://github.com/ValveSoftware/openvr/blob/master/headers/openvr_capi.h)
- [Valve OpenVR API documentation](https://github.com/ValveSoftware/openvr/wiki/API-Documentation)

`Cargo.lock` records the dependency versions. At the time of this investigation, Linux build instructions had been prepared. Linux package builds were verified later.
