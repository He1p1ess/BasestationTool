# Bestandsanalyse und Architektur

Stand: 19.10.2026. Sämtliche Dateien unter `Logs`, `Screenshots` und `BaseStation Tool Old` wurden ausschließlich gelesen. SHA-256-Inventar: `reference-hashes.json`. Die alte Anwendung wurde nicht gestartet.

## Referenzen

- Beide Screenshots: kompakte WPF-Tabelle, Statusabfrage, Scan/Reset, Einzel-/Gesamtsteuerung, Kanalwahl, Diagnose, Logexport, Sprachauswahl. Eine separate Checkbox-Mehrfachauswahl ist dort noch nicht vorhanden.
- `Logs/log.txt`: drei Geräte Station 3, Station 2 und Station 1; Adressen (anonymisiert), (anonymisiert) und (anonymisiert). Valve Corp., Modell 1004. Beobachtete Kanäle 1, 9, 5; Power-Werte 00 und 0B. Sechs Services pro Diagnose.
- Standardservices 1800, 1801, 180A; 180A enthält Hersteller 2A29, Modell 2A24, Seriennummer 2A25, Hardware 2A27, Firmware 2A26. Proprietäre zusätzliche Services und DFU-Service FE59 werden nur diagnostisch aufgelistet. Keine abgeleiteten Schreibbefehle.
- EXE: 238.449.392 Bytes, .NET-8-Single-File-Bundle, Windows x64, WPF. ILSpy 11.1 kann das enthaltene `Basestation Tool.dll` statisch als IL ausgeben. Die R2R-Variante konnte mit dem C#-Decompiler wegen einer Metadaten-/Resolver-Ausnahme nicht vollständig dekompiliert werden. Das IL der regulären Assembly ist lesbar und liegt als lokales Analyseartefakt unter `tools/analysis`.
- Die separat mitgelieferten DLLs sind Microsoft-WPF-/Grafik-/VC-Runtime-Komponenten, keine eigene BLE-Implementierung. Das PDB ist als Debugsymbol-Datei vorhanden; keine zusätzlichen Quelltexte neben dem Bundle gefunden.

## Evidenz nach Herkunft

| Information | Nachweis |
|---|---|
| Service 1523, Power 1525, Channel 1524 | Nutzer-Codefragmente, Logs und IL |
| Power schreiben 00/02/**01**, WriteWithoutResponse | Korrektur 20.10.: IL der Aufrufer `btnModeOn_Click` und `btnModeOnAll_Click` lädt `ldc.i4.1`; `SendPowerCommandAsync` übergibt dieses Byte unverändert und GattWriteOption=1. Die ursprüngliche Angabe 0B war ein Lesewert, kein Einschaltbefehl. |
| Kanal 1–16, WriteWithResponse | Nutzer-Codefragmente; Kanal-IL enthält 1524 und WriteValueAsync |
| Status über Power-Characteristic | **Zusätzlich im IL bestätigt**: UUID 1525 gefolgt von ReadValueAsync(CacheMode=1/Uncached), ungefähr Zeilen 3451–3500 |
| Kanal lesen über 1524 | IL ungefähr Zeilen 3589–3638 |
| Status 00/0B | In Logs beobachtet |
| Standby 02 | Durch Nutzer-Code belegt; nicht als Hardwaremessung dieser Entwicklung ausgegeben |
| Unbekannte Statuswerte als On | Fehler der Altlogik laut Auftrag; neue Implementierung erhält Unknown(Byte) |
| WriteWithoutResponse-Eigenschaft | In alten Logs **nicht** ausgewiesen, nur Read/Write/Notify. Neue Anwendung verwendet den ausdrücklich bestätigten Schreibmodus, protokolliert die Abweichung und verlangt einen Hardwaretest |
| Andere Lighthouse-Generationen | Nicht verifiziert; ohne LHB-Profil keine Schreibfreigabe |

Die alte Diagnose gab Binärdaten ungeprüft als Text aus. Neu: Hex immer, Text nur bei gültigem UTF-8 ohne unerlaubte Steuerzeichen.

## Umgesetzter technischer Plan

1. Protokoll und Validierung (`protocol.rs`) unabhängig von Betriebssystem und GUI.
2. Identität, lokale Namen und Zielmengen (`device.rs`); UUIDs sind niemals Geräte-IDs.
3. Echte BLE-Kommunikation (`ble.rs`) über btleplug 0.13.5. Ein Worker mit begrenzter Befehlsqueue, höchstens eine laufende Geräteoperation. Einzelne Gruppenfehler werden protokolliert und die Gruppe fortgesetzt. Zwölf Sekunden pro BLE-Aufruf; zwei Verbindungsversuche, höchstens zwei erneute Discoveries. Keine automatischen Schreibwiederholungen.
4. Native GUI (`gui.rs`) über eframe/egui 0.37, Tabelle mit anpassbaren Spalten, DE/EN, System/Dark/Light, von Checkboxen unabhängige Einzelgeräteauswahl.
5. Separate Konfiguration, Logrotation und Diagnosemodule. Fehlerhafte Konfigurationsdateien bleiben erhalten. `control.rs` trennt Power-/Kanaltransaktionen über ein Transport-Trait vom tatsächlichen BLE-Backend; Tests nutzen einen eindeutig simulierten Transport.
6. SteamVR (`steamvr.rs`): offizielles Application Manifest und IVRApplications_008 AddApplicationManifest/SetApplicationAutoLaunch/GetApplicationAutoLaunch. Registrierung nur durch explizite Bedienung. Prozessbeobachtung auf eigenem Thread, Einzelinstanz-Dateisperre.
7. Formatierung, Compilercheck, Clippy, Unit-Tests, Release-Build, GUI-Start ohne BLE, dokumentierte Hardware-Abnahme.

## Cache und Identität

Im Quellcode von btleplug 0.13.5 geprüft: Windows-Lesezugriffe verwenden Uncached; Service-/Characteristic-Discovery beginnt ebenfalls Uncached und kann bei einem Timeout gecachte Characteristic-Daten verwenden. Bereits erkannte Characteristics werden wiederverwendet; erneute Discovery bei Wiederverbindung oder fehlender UUID. Der Scan verarbeitet aktuelle Advertising-Ereignisse statt alte Adaptereinträge als neue Sichtung auszugeben. RSSI bleibt ausdrücklich ein letzter Scanwert.

Identität: Betriebssystempräfix plus `PeripheralId`, unter Windows adressbasiert, unter Linux backendabhängig. Wechselnde Adressen werden **nicht** automatisch anhand identischer Namen zusammengeführt; so werden keine Befehle versehentlich umgeleitet. Die dauerhafte Zuordnung bei Privacy-Adresswechseln bleibt eine Hardware-/Backend-Grenze.

## Nachanalyse des Einschaltfehlers (20.10.2026)

Der Besitzer meldet: Rust zeigt 0B, Station wird aber nicht grün/betriebsbereit; die alte C#-Anwendung schaltet sie ein. Statische Prüfung der tatsächlichen Button-Aufrufer bestätigt den entscheidenden Unterschied:

```text
<btnModeOn_Click>d__16.MoveNext:
IL_0012: ldc.i4.1
IL_0013: call ... MainWindow::SendPowerCommandAsync(uint8)

<btnModeOnAll_Click>d__19.MoveNext:
IL_0012: ldc.i4.1
IL_0013: call ... MainWindow::SendPowerCommandToAllAsync(uint8)
```

In `tools/analysis/Basestation Tool.il` stehen diese Aufrufe ungefähr bei Zeile 5111 und 4977. Die erste Analyse hatte nur die Schreibfunktion geprüft, aber die tatsächlichen Argumente der Button-Aufrufer übersehen. Rust schrieb deshalb fälschlich den vorgegebenen Lesewert 0B.

Die neuen unveränderten Diagnoseexporte `Logs/Log-gestartet-über-das-CSharp-Tool.txt` und `Logs/Log-gestartet-über-das-Rust-Tool.txt` unterscheiden sich im 64-Byte-Datenblock von `00000010-0060-7990-5544-1cce81af42f0` an genau einem Byte: Offset 20 (nullbasiert), Rust 0B, C# 0C. Beide Power-Reads an 1525 liefern 0B. Die zusätzliche Characteristic ist nur lesbar. Zwei Momentaufnahmen beweisen weder eine Zählerfunktion noch einen Betriebszustand dieses Bytes; es wird weiterhin roh angezeigt und nicht beschrieben.

Externe Reverse-Engineering-Evidenz bestätigt Wake-Schreibbyte 01 und die davon getrennten gemeldeten Wachwerte 01/09/0B: [Ben Woodfords GATT-Aufzeichnung](https://gist.github.com/BenWoodford/3a1e500a4ea2673525f5adb4120fd47c). Auch [lighthouse-v2-manager](https://github.com/nouser2013/lighthouse-v2-manager/blob/master/lighthouse-v2-manager.py) schreibt 01 zum Einschalten. Dies ist Community-Evidenz, keine offizielle Valve-Spezifikation. Rust erhält bei diesen drei Lesewerten das Originalbyte, zeigt „Ein gemeldet“ und behauptet damit keine physische Laser-/Motorbereitschaft. Andere Bytes bleiben Unknown.

Zur Timingfrage: Rust verarbeitet BLE-Aufträge seriell und wartet zwischen Power-Schreiben und Rücklesen eine Sekunde. Im alten IL findet sich zusätzlich eine 1500-ms-Verzögerung im Wiederholungs-/Fehlerpfad. Es gibt bislang keinen Nachweis, dass die Rust-Lesefrequenz das Einschalten verhindert. Zunächst wurde nur der belegte Befehlsfehler korrigiert; keine spekulativen Zusatzschreibvorgänge oder Schreibwiederholungen.

## Bibliotheksquellen

- https://docs.rs/btleplug/0.13.5/btleplug/api/trait.Peripheral.html
- https://docs.rs/eframe/0.37.0/eframe/
- https://github.com/ValveSoftware/openvr/blob/master/headers/openvr_capi.h
- https://github.com/ValveSoftware/openvr/wiki/API-Documentation

Bibliotheksversionen sind durch Cargo.lock festgehalten. Die Linux-Anleitung ist vorbereitet; ein Linux-Build wurde auf diesem Windows-Rechner nicht behauptet.
