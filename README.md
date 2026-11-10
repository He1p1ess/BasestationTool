# Basestation Tool (Rust)

Select your Lighthouse stations and turn them on, put them in standby or send them to sleep over Bluetooth LE. Enable automatic wake-up to switch them on when you start SteamVR.

[Download the Windows installer, portable EXE or Linux packages from GitHub Releases](https://github.com/He1p1ess/BasestationTool/releases/latest)

Most of the project code was generated with AI. Ein Großteil des Projektcodes wurde mit KI generiert.

Native Windows-11-Anwendung zur BLE-Steuerung von Valve/HTC Lighthouse Base Stations. Vollständige Rust-Neuentwicklung mit egui, ohne .NET-Laufzeit. Die Linux-Architektur verwendet denselben Anwendungskern und btleplug/BlueZ.

## Installation (Version 0.1.0)

- Windows x64: `dist/Basestation-Tool-0.1.0-x64.msi` per Doppelklick installieren. Die Installation gilt für den aktuellen Benutzer und legt einen Startmenüeintrag an. Die portable EXE bleibt unter `target/release/basestation-tool.exe` verfügbar.
- Debian/Ubuntu amd64: im Paketordner `sudo apt install ./basestation-tool_0.1.0_amd64.deb` ausführen. Anschließend über das Anwendungsmenü oder mit `basestation-tool` starten.
- Linux x86-64 ohne Installation: `dist/Basestation-Tool-0.1.0-x86_64.AppImage` an einem dauerhaften Ort ablegen, mit `chmod +x Basestation-Tool-0.1.0-x86_64.AppImage` ausführbar machen und mit `./Basestation-Tool-0.1.0-x86_64.AppImage` starten. Falls FUSE fehlt: `./Basestation-Tool-0.1.0-x86_64.AppImage --appimage-extract-and-run`. Benötigt eine grafische Linux-Sitzung, glibc ab 2.31 und die Systembibliotheken für Grafik, D-Bus und Bluetooth (BlueZ). Das Paket wurde entpackt und geprüft, aber nicht auf einem Linux-Desktop gestartet.
- Prüfsummen aller Pakete: `dist/SHA256SUMS.txt`. Das MSI ist nicht signiert. Bei zuvor eingerichtetem SteamVR-Autostart nach dem Wechsel von der portablen zur installierten EXE den Autostart in den Einstellungen erneut registrieren.

## Start

Windows-EXE: **`target/release/basestation-tool.exe`**. Doppelklick startet die Anwendung. Ein Bluetooth-LE-Adapter und aktuelle Windows-Treiber sind erforderlich. Ein zweiter Start derselben Benutzerinstanz beendet sich sofort.

Die neue Anwendung scannt standardmäßig automatisch und liest Status/Kanal. Sie sendet beim normalen Start **keine Power- oder Kanalbefehle**. Schreibaktionen benötigen eine Bestätigung mit der konkreten Zielliste. Automatische SteamVR-Poweraktionen sind standardmäßig ausgeschaltet.

## Bedienung

Sprache und „Log anzeigen“ befinden sich in den Einstellungen rechts oben beim Zahnrad. Ein Namensklick wählt genau eine Station. Checkboxen ändern die Auswahl stationenweise. Bei genau einer ausgewählten Station erscheinen zusätzlich lokaler Name, Kanal und Diagnose. Nach einem Log- oder Diagnoseexport öffnet sich automatisch der Exportordner; der vollständige Dateipfad bleibt zusätzlich sichtbar.

Ein Klick auf eine Spaltenüberschrift sortiert die Tabelle, ein weiterer kehrt die Reihenfolge um. Kanal und RSSI werden numerisch sortiert. Die ausgewählten Stationen bleiben an die Gerätekennung gebunden. Bei sichtbarem Log lässt sich die horizontale Trennlinie unter der Tabelle ziehen, um Platz zwischen Geräteliste und Log umzuverteilen. Zusätzliche Fensterhöhe vergrößert den Logbereich.

- **Suchen:** acht Sekunden BLE-Scan, danach Statusabfrage. Weitere Suche automatisch alle 60 Sekunden, abschaltbar. Statusabfragen im Leerlauf alle 30 Sekunden.
- **Tabelle:** Die markierten Zeilen zeigen die aktuelle Auswahl. Tabellen-Spaltengrenzen lassen sich ziehen; bei schmalen Fenstern horizontal scrollen.
- **Einzelgerät / ausgewählte Geräte / alle Geräte:** eine gemeinsame Aktionsleiste für Ein, Standby, Sleep und Statusabfrage. Die Namen der Zielstationen stehen direkt über den Schaltflächen. Vor dem Schreiben zeigt ein Dialog die tatsächlichen Ziele. Die Zielmenge wird beim Auslösen festgehalten.
- **Lokaler Name:** frei bearbeiten und speichern. Der Bluetooth-Name wird nicht verändert.
- **Kanal:** 1–16 für das erkannte LHB-Profil. Der tatsächlich zurückgelesene Kanal wird angezeigt, nicht einfach der gewünschte Wert.
- **Diagnose:** Services, Characteristics, Eigenschaften, Hexwerte und geprüfter Text, einschließlich Hersteller/Modell/Seriennummer/Firmware. Aktualisieren, Textauswahl/Kopieren und Gesamtexport.
- **RSSI:** letzter beim Scan beobachteter dBm-Wert; Zeitpunkt im Tooltip. Ein fehlender Wert bleibt „—“. Keine Entfernungsberechnung und keine erfundenen Signalwerte.
- **Bluetooth zurücksetzen:** beendet nur programminterne Verbindungen und initialisiert den Scan neu. Kein Deaktivieren des Adapters, keine Windows-Einstellungsänderung.
- **Log:** Live-Anzeige mit Suche, Level-Filter, Clear und Save/Export. Technische Log-/GATT-Inhalte bleiben Englisch. BLE-Debug zeigt Bytewerte. Maximal 2.000 Einträge im Speicher; aktuelle Datei und vier rotierende Vorgänger mit ungefähr 2 MB je Datei.
- **Einstellungen:** Deutsch, Englisch, Französisch, Spanisch und Italienisch; System/Dunkel/Hell, automatischer Scan, Logging, SteamVR. Fenstergröße/-position speichert eframe automatisch.

## SteamVR

Die Anwendung verwendet die offizielle OpenVR-Registrierung: Application Manifest, `AddApplicationManifest`, `SetApplicationAutoLaunch`, anschließend `GetApplicationAutoLaunch`. Es werden keine Registry-Run-Einträge oder systemweiten Aufgaben eingerichtet.

1. EXE an ihrem dauerhaften Ort ablegen und SteamVR starten.
2. In Einstellungen **Automatisch mit SteamVR starten** aktivieren. Erfolg oder tatsächliche OpenVR-Fehler werden angezeigt; erst nach bestätigter Registrierung wird die Einstellung gespeichert.
3. Zum Abschalten bei laufendem SteamVR denselben Haken entfernen.

Die offizielle OpenVR-Bibliothek wird über `openvrpaths.vrpath` gefunden. Alternativ kann ihr absoluter Pfad angegeben werden, unter Windows üblicherweise `SteamVR/bin/win64/openvr_api.dll`. Es wird keine OpenVR-DLL aus dem Internet nachgeladen. Wird die EXE verschoben, Registrierung erneut aktivieren.

**Mit SteamVR beenden** wirkt nur, wenn das Tool mit `--steamvr` durch das Manifest gestartet wurde. Eine manuell gestartete Instanz bleibt offen. Ein separater Monitor beobachtet `vrserver`, ohne SteamVR selbst zu starten.

Die Optionen „Bei SteamVR-Start einschalten“ und „Beim SteamVR-Ende Standby/Sleep“ autorisieren automatische echte Powerbefehle an bekannte Stationen. Die Endaktion ist eine exklusive Auswahl (keine Aktion/Standby/Sleep). Alle Automatik-Poweraktionen sind anfangs deaktiviert. Beim automatischen Schließen werden zuvor eingereihte Aktionen abgewartet.

Die OpenVR-Registrierung wurde am 20.10.2026 mit SteamVR 2.17.10 tatsächlich aktiviert, zurückgelesen und wieder deaktiviert. Der anschließende SteamVR-Neustart scheiterte am fehlenden Headset (`HmdNotFound`, 108); der tatsächliche Autostart bleibt daher bei betriebsbereiter VR-Sitzung zu prüfen. Das Manifest nutzt die Overlay-Registrierung; das Programm stellt ein Desktopfenster, kein VR-Overlay dar.

## Dateien und Einstellungen

Benutzerbezogenes Konfigurationsverzeichnis über `directories::ProjectDirs`:

- Windows: `%APPDATA%\BasestationTool\BasestationTool\config` (der tatsächlich verwendete Pfad steht in Einstellungen).
- Linux: gewöhnlich `~/.config/basestationtool`.

`settings.json` enthält bekannte IDs, Namen und Einstellungen. Vor dem Ersetzen entsteht `settings.previous.json`. Ungültiges JSON wird nicht überschrieben; die Oberfläche zeigt einen Fehler und deaktiviert das Speichern, bis die Datei außerhalb der Anwendung korrigiert wurde. Laufzeitstatus wird nicht als vermeintlich aktueller Zustand gespeichert. Bekannte Geräte starten offline.

Neue Logs liegen unter `logs/`, Exporte unter `exports/` dieses Benutzerverzeichnisses. **Die alten Referenzdateien im Projekt bleiben unverändert.** Geräteadressen/Seriennummern können in Diagnosen und Logs enthalten sein.

Für isolierte Tests kann `BASESTATION_DATA_DIR` ein anderes Datenverzeichnis festlegen. `--no-ble` verhindert Scan, Lesen und Schreiben vollständig; das Fenster kennzeichnet diesen Modus. Es werden dabei keine Geräte simuliert. Simulationen existieren ausschließlich im Unit-Test-Transport.

## Windows-Build

Voraussetzungen: Rust stable (hier Rust/Cargo 1.99.1), MSVC x64, Visual Studio Build Tools mit Desktopentwicklung C++ und Windows SDK. Cargo muss die in Cargo.lock festgelegten Abhängigkeiten herunterladen können.

```powershell
cargo fmt --check
cargo check --locked
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo build --locked --release
.\target\release\basestation-tool.exe
```

Bei fehlendem Cargo im PATH: `%USERPROFILE%\.cargo\bin\cargo.exe` verwenden. Zum Ausführen der fertigen Anwendung sind weder Cargo noch .NET erforderlich; die üblichen Windows-/Grafiktreiber und gegebenenfalls die MSVC-Runtime müssen vorhanden sein.

## Linux

Linux-x86-64 wurde auf Windows für GNU/Linux mit glibc 2.31 als Buildziel kompiliert. Das Debian-Paket und seine ELF-Datei wurden geprüft; Start der GUI, BlueZ und SteamVR sind auf Linux noch nicht praktisch getestet. Der gemeinsame Code nutzt plattformübergreifende Bibliotheken; native Abhängigkeiten müssen auf Linux installiert sein, z. B. unter Debian/Ubuntu:

```sh
sudo apt install build-essential pkg-config libdbus-1-dev libudev-dev libx11-dev libxi-dev libxrandr-dev libxcursor-dev libxinerama-dev libxkbcommon-dev libwayland-dev libegl1-mesa-dev bluez
cargo build --locked --release
./target/release/basestation-tool
```

BlueZ/D-Bus müssen laufen und dem angemeldeten Benutzer Bluetooth-Zugriff erlauben (distributionsabhängige Policy). Die GUI nicht als root starten. Bei fehlenden Berechtigungen oder Adapterfehlern werden reale Backendfehler angezeigt. SteamVR benötigt eine Linux-Runtime mit `bin/linux64/libopenvr_api.so` und muss separat überprüft werden.

## Architektur und Protokoll

| Modul | Aufgabe |
|---|---|
| `gui` | Oberfläche, Zielauswahl, Bestätigung, Anzeige |
| `device` | stabile Identitäten und Zielmengen |
| `ble` | tatsächlicher Adapter, Discovery, begrenzte Queue/Timeouts |
| `protocol` | UUIDs, typsichere Befehle, Status-/Kanalvalidierung |
| `control` | unabhängig testbare Schreib-/Rücklesetransaktionen |
| `diagnostics` | Hex-/Textvalidierung und Standardinformationen |
| `config` | dauerhafte Benutzereinstellungen |
| `logging` | begrenzte Anzeige, Dateirotation und Exporte |
| `steamvr` | OpenVR-ABI und Prozessüberwachung |

Power-Service `00001523-1212-efde-1523-785feabcd124`, Power-Characteristic `00001525-1212-efde-1523-785feabcd124`: Sleep `00`, Standby `02`, On **`01`**, genau ein Byte ohne GATT-Antwort. Korrektur nach Prüfung der tatsächlichen Button-Aufrufer in der alten EXE: `0B` ist ein gelesener Wert, nicht der Einschaltbefehl. Die gemeldeten Wachwerte `01`, `09`, `0B` werden mit Rohbyte als „Ein gemeldet“ angezeigt; dies bestätigt keine physische Laser-/Motorbereitschaft. Nach einer Sekunde wird gelesen, maximal drei Leseversuche. Channel-Characteristic `00001524-1212-efde-1523-785feabcd124`: Byte 1–16 mit GATT-Antwort, danach Rücklesen. Schreibbefehle werden nicht automatisch erneut gesendet. Andere Statusbytes bleiben Unknown.

Nur das aus den Referenzen bekannte LHB-Profil wird beschreibbar. Andere Generationen und wechselnde Privacy-Adressen sind nicht automatisch kompatibel. Die Identität verwendet Betriebssystempräfix und Backend-PeripheralId, niemals eine Service-UUID oder einen lokalen Namen. Identitätswechsel werden nicht automatisch zusammengeführt.

## Abnahme und bekannte Grenzen

Siehe `docs/ANALYSE.md` für Quellen/Evidenz und `docs/TESTS.md` für Prüfungen und offene Hardwaretests. Die Windows-Release-EXE wurde an drei echten LHB-Stationen geprüft.

- Einzel-, Auswahl- und Gesamtsteuerung sowie Kanäle 1/5/16 wurden nach Freigabe real geschrieben und zurückgelesen. **Die damalige On-Prüfung war kein physischer Funktionsnachweis:** Der Besitzer meldete fehlende Betriebsbereitschaft trotz 0B. Das Einschaltbyte wurde inzwischen auf das in der alten EXE belegte 01 korrigiert; ein erneuter physischer Einschaltcheck steht aus. Sleep/Standby funktionieren laut Besitzer.
- Power-Schreiben ohne GATT-Antwort funktioniert an den drei geprüften Stationen trotz fehlender WriteWithoutResponse-Eigenschaft. Andere Geräte-/Treiberkombinationen bleiben separat zu prüfen.
- Ein Adapter wird automatisch ausgewählt. Eine Mehradapter-Auswahl ist derzeit nicht enthalten.
- Spalten sind anpassbar, jedoch nicht einzeln ausblendbar. Keine automatische Zusammenführung wechselnder BLE-Adressen.
- SteamVR-Registrierung erfolgreich getestet und wieder deaktiviert; tatsächlicher Autostart, Ende-/Power-Automatik und Linux-Laufzeit benötigen weitere praktische Abnahme.

Primärquellen: [btleplug API](https://docs.rs/btleplug/0.13.5/btleplug/api/trait.Peripheral.html), [eframe API](https://docs.rs/eframe/0.37.0/eframe/), [Valve OpenVR Header](https://github.com/ValveSoftware/openvr/blob/master/headers/openvr_capi.h).
