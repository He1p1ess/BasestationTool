# Prüfbericht

Weitere Layoutprüfung am 20.10.: Sprache und Log-Sichtbarkeit in Einstellungen verschoben; Log-Sichtbarkeit wird gespeichert. Diagnose unter Kanal. Alle Tabellenüberschriften sortierbar, Richtung umschaltbar; IDs bleiben Grundlage der Auswahl. Ziehbare horizontale Trennlinie und Log ohne feste Höhenobergrenze. Clippy und Release-Build erfolgreich. In isolierter `--no-ble`-Instanz Namenssortierung in beide Richtungen, Gerätefokus, Einstellungen und Ziehen am mittleren Griff sichtbar geprüft. Beim Maximieren blieb die gewählte Tabellenhöhe erhalten und der Logbereich erhielt die zusätzliche Höhe. Keine Hardwareaktionen.

## Oberflächenüberarbeitung am 20.10.2026

Nach Bestätigung des Besitzers, dass Einschalten mit der korrigierten Version funktioniert: Sprach-Combobox, lesbare Auswahlüberschrift, gemeinsamer Einzelgerätbereich mit Name/Adresse, einheitliche Buttonhöhen und Einstellungen rechts oben mit gezeichnetem Zahnrad umgesetzt. Release-Build und Clippy aller Standardtargets ohne Warnungen. Sichtprüfung mit `--no-ble` und separater Konfiguration unter `target/ui-layout-test`: Gerätefokus, Deutsch/Englisch-Combobox, Einzelgerätbereich und Einstellungsfenster geprüft. Log exportiert und neu geöffnetes Explorer-Fenster für den korrekten Test-Exportordner bestätigt. Keine Hardwarebefehle gesendet, normale Benutzerkonfiguration nicht geändert. Die folgende frühere Einschaltanalyse bleibt als Verlauf erhalten; der physische Einschalterfolg ist inzwischen vom Besitzer bestätigt.

**Korrektur nach Nutzerbeobachtung am 20.10.2026:** Die frühere Abnahme bestätigte nur zurückgelesene Werte. Sie belegte ausdrücklich nicht die physische Einschaltfunktion; der Besitzer meldete anschließend fehlende grüne LED/Betriebsbereitschaft trotz 0B. Die Formulierung „Hardwaretests bestanden“ war für das Einschalten zu weitgehend. Die alte EXE sendet nach erneuter IL-Prüfung 01 zum Einschalten, während Rust bisher 0B sendete. Der Fehler ist korrigiert; die neue EXE ist noch nicht physisch als betriebsbereit bestätigt. Details: `ANALYSE.md`, Abschnitt Nachanalyse.

Aktuelle Softwareprüfung nach dieser Korrektur: **14 Tests bestanden**, Clippy mit allen Targets und `hardware-tests` ohne Warnungen, Release-Build erfolgreich. Neuer Regressionstest: Wake schreibt immer 01 ohne GATT-Antwort, akzeptiert gemeldete 01/09/0B unter Erhalt des Rohbytes und lässt 0C unbekannt. Es wurden bei dieser Korrektur keine neuen Power-/Kanalbefehle gesendet. Die untenstehenden Hardwarelogs bleiben unveränderte historische Nachweise der vorherigen Version.

Windows x64, 19.–20.10.2026, Rust/Cargo 1.99.1. Keine Ausführung der alten EXE. Alle zehn Referenzdateien abschließend am 20.10. per SHA-256 verglichen: unverändert.

## Automatisierte Prüfungen

- `cargo fmt --check`: erfolgreich.
- `cargo check --locked`: erfolgreich.
- `cargo clippy --locked --all-targets --features hardware-tests -- -D warnings`: abschließend erfolgreich, keine Warnungen; Standardbuild ebenfalls zuvor geprüft.
- `cargo test --locked`: **13 Tests bestanden**, kein Fehlschlag.
- `cargo build --release`: erfolgreich, native Windows-EXE unter `target/release/basestation-tool.exe` (ca. 8,9 MB).

Abgedeckt: sämtliche vorgegebenen Powerbytes und Statuswerte, Unknown/ungültige Antwortlängen, Kanal 1/5/16 und Grenzen, eindeutige Identität trotz gleicher/änderbarer Anzeigenamen, Einzel-/Auswahl-/Gesamtzielmengen, Name-/Einstellungs-Serialisierung, deaktivierte Automatik als Standard, begrenzter Logspeicher, Binär-/Texttrennung, Fehlerisolierung je Gerät, Power-Schreibart ohne Antwort, Kanal-Schreibart mit Antwort, verzögerte Rücklesebestätigung, ausbleibende Bestätigung trotz erfolgreichem Schreiben und tatsächlicher statt gewünschter Kanal.

Die Transporttests verwenden explizit `SimulatedTransport`, keine Hardware. Der Backend-Fehlerisolationstest verwendet fehlende simulierte IDs, ohne einen Adapter zu öffnen.

## GUI-Test

Release-EXE tatsächlich auf Windows gestartet. Zunächst mit `--no-ble` und isoliertem Datenverzeichnis:

- Hauptfenster und Einstellungen sichtbar und bedienbar.
- Dark Mode und Light Mode visuell geprüft; Kontrast der Warnmeldung korrigiert.
- Deutsch/Englisch umgeschaltet.
- Sprache und Theme nach Programmneustart aus Testkonfiguration wiederhergestellt.
- Drei klar benannte lokale `SIMULATED-*`-Testzeilen geladen. Sie blieben offline; kein RSSI/Status wurde erfunden.
- Checkbox für Testgerät A und Einzelgerätefokus für Testgerät B unabhängig gewählt; beide Zielzähler korrekt.
- Beenden über Fensterschließen erfolgreich.

Testdaten liegen ausschließlich unter `target/ui-test-data`, nicht im normalen Benutzerdatenverzeichnis. Die Oberfläche enthält keinen simulierten Livebetrieb.

## Tatsächlich ausgeführter BLE-Lesetest

Die Release-EXE wurde zusätzlich mit einer separaten Konfiguration unter `target/hardware-read-test` normal gestartet. Automatische Poweraktionen waren explizit ausgeschaltet. Es wurden ausschließlich Scan, Verbindung, Discovery und Read ausgeführt:

| Gerät | Adresse | Gelesener Status | Gelesener Kanal |
|---|---|---|---|
| Station 1 | (anonymisiert) | Sleep / 00 | 5 / 05 |
| Station 2 | (anonymisiert) | Sleep / 00 | 9 / 09 |
| Station 3 | (anonymisiert) | Sleep / 00 | 1 / 01 |

Erster Scan 22:09:13–22:09:21, alle Statusabfragen bis 22:09:29 abgeschlossen. Wiederholte Status-/Kanallesung um 22:09:43 erfolgreich. Damit sind echte Windows-BLE-Erkennung, GATT-Discovery und diese Lesewege bestätigt. Die später freigegebenen Schreibtests stehen unten.

Zusätzlich wurden am Gerät Station 1 um 22:11:20–22:11:22 die lesbaren GATT-Characteristics einschließlich Modell, Seriennummer, Firmware und Hersteller erfolgreich diagnostiziert. Der Diagnoseexport wurde über die Oberfläche erzeugt und als Datei geprüft. Reale RSSI-Werte wurden angezeigt (beispielsweise −58/−66/−52 dBm beim zweiten Scan, keine Entfernungsinterpretation).

Beim Live-Test wurde eine gemeinsame Scroll-ID von Tabelle und Log erkannt und durch eindeutige IDs behoben. Die reale Gerätetabelle wurde danach erneut sichtbar geprüft. Vollständige Format-/Check-/Clippy-/Test-/Release-Prüfung nach der letzten Änderung erneut erfolgreich.

Nachweise: `hardware-read-test.log` und `hardware-diagnosis.txt` in diesem Verzeichnis. Der Log enthält keine `TX`-Einträge. Testinstanz abschließend geschlossen; keine SteamVR-Registrierung verändert.

## Freigegebene Hardware-Abnahme am 20.10.2026

Nach ausdrücklicher Freigabe des Besitzers wurden reale BLE-Schreibtests durchgeführt. Der separate Runner verwendet den tatsächlichen Backend-/Transportcode und die Zielauswahl der Anwendung. Er ist nur mit Cargo-Feature `hardware-tests` und Argument `--hardware-acceptance` enthalten; die ausgelieferte normale Release-EXE enthält ihn nicht.

- Einzelgerät, zwei ausgewählte Geräte und alle drei Geräte: jeweils On, Standby und Sleep, insgesamt neun Zielmengen-/Befehlskombinationen bestanden.
- Nach jedem Schritt reale Status-/Kanallesungen sämtlicher Geräte; nicht ausgewählte Geräte unverändert.
- Power-Schreiben ohne GATT-Antwort tatsächlich erfolgreich, obwohl die gemeldeten Eigenschaften kein WriteWithoutResponse nennen.
- Kanäle 1, 5 und 16 an Station 1 mit GATT-Antwort geschrieben und jeweils korrekt zurückgelesen.
- Zwischenzeitlich gelesener Powerwert `09` korrekt als Unknown behandelt, ohne seine Bedeutung zu erfinden.
- Ausgangswerte vor dem ersten Schreiben gespeichert und nach dem Test um 15:17:34 wiederhergestellt und zurückgelesen:

| Station | Wiederhergestellter Zustand | Kanal |
|---|---|---|
| Station 1 | On | 5 |
| Station 2 | Sleep | 9 |
| Station 3 | Sleep | 1 |

Nachweise: `hardware-acceptance-20261020.log`, `hardware-baseline-20261020.json`. Ergebnis `ACCEPTANCE RESULT: Ok(())`. Bestätigt sind BLE-Antworten; eine unabhängige Beobachtung von Motor/Laser fand nicht statt.

## Freigegebener SteamVR-Test am 20.10.2026

SteamVR 2.17.10 gestartet. Im ersten Versuch deckte Fehler 108 einen falschen Initialisierungstyp auf: OpenVR Background (3) statt Utility (4). Der Code verwendet jetzt ausdrücklich Utility, ohne HMD-/Treiberinitialisierung.

Danach Autostart über die GUI aktiviert: AddApplicationManifest, SetApplicationAutoLaunch und GetApplicationAutoLaunch erfolgreich, Einstellung nach Tool-Neustart erhalten. Tool und SteamVR geschlossen, anschließend SteamVR erneut gestartet. Der Runtime-Start scheiterte an `VRInitError_Init_HmdNotFound` (108); kein Tool-Autostart beobachtet. Damit ist die Registrierung bestätigt, der tatsächliche Autostart bei betriebsbereiter VR-Sitzung bleibt offen.

Anschließend Autostart über die GUI wieder deaktiviert und per OpenVR zurückgeprüft. SteamVR-Datei `org.basestationtool.desktop.vrappconfig` bestätigt `autolaunch: false`, `last_launch_time: "0"`. Automatische Poweraktionen und Schließen mit SteamVR blieben ausgeschaltet. SteamVR und Tool abschließend geschlossen. Die Stationen waren zuletzt weiterhin On/5, Sleep/9 und Sleep/1.

Nachweise: `steamvr-runtime-test-20261020.log` (relevante Runtime-Auszüge), `steamvr-restored-20261020.json` (deaktivierter Autostart).

## Offene praktische Prüfungen

- Tatsächlicher SteamVR-Autostart bei verbundenem Headset, Instanzschutz bei diesem Startweg und optionales automatisches Schließen.
- Automatische SteamVR-Poweraktionen nur nach separater Aktivierung; bisher nicht real getestet.
- Physischer Funk-/Stromausfall, ausgeschalteter Adapter, fehlender Service und reale Timeouts; Fehlerisolierung bisher zusätzlich automatisiert mit simuliertem Transport geprüft.
- Aliaszuordnung mit realem Gerät über Neustart; Serialisierung und stabile Identitäten automatisiert geprüft.
- Linux-Build/-Ausführung und andere Lighthouse-Generationen.

