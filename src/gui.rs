use crate::i18n::{Language, strings as s};
use crate::{
    ble::{Event, Request, Worker},
    config::Config,
    device::{self, Action, Device, Scope},
    logging::Logs,
    protocol::{Channel, PowerCommand, Status},
    steamvr,
};
use eframe::egui::{self, Color32, RichText};
use egui_extras::{Column, TableBuilder};
use std::{
    collections::{BTreeMap, BTreeSet},
    time::{Duration, Instant},
};
pub struct App {
    config: Config,
    devices: BTreeMap<String, Device>,
    selected: BTreeSet<String>,
    worker: Worker,
    logs: Logs,
    busy: bool,
    queued: usize,
    settings: bool,
    diagnosis: Option<String>,
    channel: u8,
    search: String,
    level: String,
    show_logs: bool,
    pending: Option<(Vec<String>, Action)>,
    notice: String,
    monitor: steamvr::Monitor,
    steam_running: bool,
    steam_initialized: bool,
    steam_seen: bool,
    pending_steam_on: bool,
    steam_launch: bool,
    close_pending: bool,
    registration: Option<std::sync::mpsc::Receiver<Result<bool, String>>>,
    last_scan: Instant,
    last_refresh: Instant,
    no_ble: bool,
    save_allowed: bool,
    table_height: f32,
    sort_column: usize,
    sort_descending: bool,
}
impl App {
    pub fn new(no_ble: bool) -> Self {
        let loaded = Config::load();
        let mut logs = Logs::default();
        let save_allowed = loaded.is_ok();
        let config = loaded.unwrap_or_else(|e| {
            logs.push("Error", format!("{e:#}"), false);
            Config::default()
        });
        let devices = config
            .devices
            .iter()
            .map(|k| {
                (
                    k.id.clone(),
                    Device {
                        known: k.clone(),
                        ..Default::default()
                    },
                )
            })
            .collect();
        let worker = Worker::start();
        let busy = !no_ble && config.auto_scan;
        if busy {
            let _ = worker.tx.try_send(Request::Scan);
        }
        logs.push("Info", "Basestation Tool started", config.auto_log);
        let show_logs = config.show_logs;
        Self {
            config,
            devices,
            selected: BTreeSet::new(),
            worker,
            logs,
            busy,
            queued: usize::from(busy),
            settings: false,
            diagnosis: None,
            channel: 1,
            search: String::new(),
            level: String::new(),
            show_logs,
            pending: None,
            notice: String::new(),
            monitor: steamvr::Monitor::start(),
            steam_running: false,
            steam_initialized: false,
            steam_seen: false,
            pending_steam_on: false,
            steam_launch: std::env::args().any(|a| a == "--steamvr"),
            close_pending: false,
            registration: None,
            last_scan: Instant::now(),
            last_refresh: Instant::now(),
            no_ble,
            save_allowed,
            table_height: 180.0,
            sort_column: 1,
            sort_descending: false,
        }
    }
    fn persist(&mut self) {
        if !self.save_allowed {
            return;
        }
        self.config.devices = self.devices.values().map(|d| d.known.clone()).collect();
        self.config.show_logs = self.show_logs;
        if let Err(e) = self.config.save() {
            self.logs
                .push("Error", format!("Settings: {e:#}"), self.config.auto_log);
            self.notice = e.to_string();
        }
    }
    fn send(&mut self, request: Request) {
        if self.no_ble {
            self.notice = "BLE disabled (--no-ble)".into();
            return;
        }
        match self.worker.tx.try_send(request) {
            Ok(()) => {
                self.queued += 1;
                self.busy = true;
            }
            Err(e) => self.notice = format!("Queue: {e}"),
        }
    }
    fn action(&mut self, ids: Vec<String>, action: Action) {
        if ids.is_empty() {
            return;
        }
        if action.writes() {
            self.pending = Some((ids, action));
        } else {
            self.send(Request::Run(ids, action));
        }
    }
    fn export(&mut self, prefix: &str, text: &str) {
        self.notice = match crate::logging::export(prefix, text) {
            Ok(path) => {
                let mut notice = format!(
                    "{}: {}",
                    self.config.language.text(s::EXPORTED),
                    path.display()
                );
                if let Err(error) = crate::logging::open_export_folder(&path) {
                    notice.push_str(&format!(
                        " — {}: {error}",
                        self.config.language.text(s::COULD_NOT_OPEN_FOLDER)
                    ));
                }
                notice
            }
            Err(e) => e.to_string(),
        };
    }
    fn events(&mut self, ctx: &egui::Context) {
        while let Ok(event) = self.worker.rx.try_recv() {
            match event {
                Event::Busy(b) => {
                    if !b {
                        self.queued = self.queued.saturating_sub(1);
                    }
                    self.busy = self.queued > 0;
                }
                Event::Log(level, message) => self.logs.push(
                    level,
                    message,
                    self.config.auto_log && (self.config.debug || level != "Debug"),
                ),
                Event::Device(mut d) => {
                    if let Some(old) = self.devices.get(&d.known.id) {
                        d.known.alias = old.known.alias.clone();
                    }
                    self.devices.insert(d.known.id.clone(), d);
                }
            }
        }
        while let Ok(running) = self.monitor.rx.try_recv() {
            let was_running = self.steam_running;
            self.steam_running = running;
            self.logs.push(
                "Info",
                format!("SteamVR running: {running}"),
                self.config.auto_log,
            );
            if !self.no_ble
                && running
                && !was_running
                && self.config.steam_on
                && (self.steam_initialized || self.steam_launch)
            {
                // Discovery must finish before resolving the automatic target set.
                // On a first launch the GUI may not have received any devices yet.
                self.pending_steam_on = true;
                if !self.busy {
                    self.send(Request::Scan);
                }
            }
            if !running && was_running && self.steam_seen {
                self.pending_steam_on = false;
                if !self.no_ble
                    && let Some(command) = self.config.steam_exit
                {
                    let ids = device::targets(&self.devices, Scope::All);
                    self.send(Request::Run(ids, Action::Power(command)));
                }
                self.close_pending = self.config.close_with_steam && self.steam_launch;
            }
            self.steam_seen |= running;
            self.steam_initialized = true;
        }
        if self.pending_steam_on && !self.config.steam_on {
            self.pending_steam_on = false;
        }
        if self.pending_steam_on && self.steam_running && !self.busy {
            self.pending_steam_on = false;
            let ids = device::targets(&self.devices, Scope::All);
            self.send(Request::Run(ids, Action::Power(PowerCommand::On)));
        }
        if self.close_pending && !self.busy {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        if let Some(rx) = &self.registration
            && let Ok(result) = rx.try_recv()
        {
            match result {
                Ok(enabled) => {
                    self.config.steam_autostart = enabled;
                    self.notice = self
                        .config
                        .language
                        .text(s::STEAMVR_AUTOSTART_VERIFIED)
                        .into();
                    self.persist();
                }
                Err(e) => {
                    self.notice = e.clone();
                    self.logs.push("Error", e, self.config.auto_log);
                }
            }
            self.registration = None;
        }
        if !self.no_ble && !self.busy && self.pending.is_none() {
            if self.config.auto_scan && self.last_scan.elapsed() > Duration::from_secs(60) {
                self.last_scan = Instant::now();
                self.send(Request::Scan);
            } else if self.last_refresh.elapsed() > Duration::from_secs(30)
                && !self.devices.is_empty()
            {
                self.last_refresh = Instant::now();
                self.send(Request::Run(
                    device::targets(&self.devices, Scope::All),
                    Action::Refresh,
                ));
            }
        }
        ctx.request_repaint_after(Duration::from_millis(250));
    }
    fn controls(&mut self, ui: &mut egui::Ui, ids: &[String]) {
        let lang = self.config.language;
        ui.horizontal_wrapped(|ui| {
            if ids.is_empty() {
                ui.label(lang.text(s::NO_STATIONS_SELECTED_CLICK_A_NAME_OR_TICK_STATIONS));
            } else {
                ui.strong(format!("{} ({}):", lang.text(s::SELECTED), ids.len()));
                for id in ids {
                    let d = &self.devices[id];
                    let name = d.display_name();
                    let label = if self
                        .devices
                        .values()
                        .any(|other| other.known.id != *id && other.display_name() == name)
                    {
                        format!("{name} · {}", d.known.address)
                    } else {
                        name.to_owned()
                    };
                    ui.label(label).on_hover_text(id);
                }
            }
        });
        ui.horizontal_wrapped(|ui| {
            ui.add_enabled_ui(!self.busy && !ids.is_empty() && !self.no_ble, |ui| {
                for (command, label) in [
                    (PowerCommand::On, lang.text(s::ON)),
                    (PowerCommand::Standby, lang.text(s::STANDBY)),
                    (PowerCommand::Sleep, lang.text(s::SLEEP)),
                ] {
                    if ui.button(label).clicked() {
                        self.action(ids.to_vec(), Action::Power(command));
                    }
                }
                if ui.button(lang.text(s::REFRESH_STATUS)).clicked() {
                    self.action(ids.to_vec(), Action::Refresh);
                }
            });
        });
    }
    fn settings(&mut self, ctx: &egui::Context) {
        if !self.settings {
            return;
        }
        let lang = self.config.language;
        let mut open = true;
        egui::Window::new(lang.text(s::SETTINGS))
            .id(egui::Id::unique("settings-window"))
            .open(&mut open)
            .default_width(530.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(lang.text(s::LANGUAGE));
                    let previous_language = self.config.language;
                    egui::ComboBox::from_id_salt("language")
                        .selected_text(lang.name())
                        .show_ui(ui, |ui| {
                            for choice in Language::ALL {
                                ui.selectable_value(
                                    &mut self.config.language,
                                    choice,
                                    choice.name(),
                                );
                            }
                        });
                    if previous_language != self.config.language {
                        self.persist();
                    }
                });
                ui.checkbox(&mut self.show_logs, lang.text(s::SHOW_LOG));
                ui.horizontal(|ui| {
                    ui.label(lang.text(s::APPEARANCE));
                    for (value, label) in [
                        (0, lang.text(s::SYSTEM)),
                        (1, lang.text(s::DARK)),
                        (2, lang.text(s::LIGHT)),
                    ] {
                        ui.radio_value(&mut self.config.theme, value, label);
                    }
                });
                ui.checkbox(
                    &mut self.config.auto_scan,
                    lang.text(s::AUTOMATICALLY_SCAN_EVERY_60_SECONDS),
                );
                ui.checkbox(
                    &mut self.config.auto_log,
                    lang.text(s::AUTOMATICALLY_SAVE_LOGS),
                );
                ui.checkbox(&mut self.config.debug, lang.text(s::BLE_DEBUG_OUTPUT));
                ui.separator();
                ui.strong("SteamVR");
                ui.label(lang.text(s::START_STEAMVR_BEFORE_CHANGING_REGISTRATION));
                let mut auto = self.config.steam_autostart;
                if ui
                    .add_enabled(
                        self.registration.is_none(),
                        egui::Checkbox::new(
                            &mut auto,
                            lang.text(s::AUTOMATICALLY_START_WITH_STEAMVR),
                        ),
                    )
                    .changed()
                {
                    let (tx, rx) = std::sync::mpsc::channel();
                    let custom = self.config.openvr_library.clone();
                    self.registration = Some(rx);
                    std::thread::spawn(move || {
                        let _ = tx.send(
                            steamvr::register(auto, &custom)
                                .map(|()| auto)
                                .map_err(|e| format!("{e:#}")),
                        );
                    });
                }
                if self.registration.is_some() {
                    ui.spinner();
                }
                ui.label(lang.text(s::OPTIONAL_ABSOLUTE_PATH_TO_THE_OFFICIAL_OPENVR_LIBRARY));
                ui.text_edit_singleline(&mut self.config.openvr_library);
                ui.checkbox(
                    &mut self.config.close_with_steam,
                    lang.text(s::CLOSE_WITH_STEAMVR_ONLY_WHEN_LAUNCHED_BY_STEAMVR),
                );
                ui.separator();
                ui.label(lang.text(
                    s::THESE_OPTIONS_AUTHORIZE_REAL_AUTOMATIC_POWER_COMMANDS_TO_ALL_KNOWN_STATIONS,
                ));
                ui.checkbox(
                    &mut self.config.steam_on,
                    lang.text(s::POWER_ON_WHEN_STEAMVR_STARTS),
                );
                ui.horizontal_wrapped(|ui| {
                    ui.label(lang.text(s::WHEN_STEAMVR_EXITS));
                    ui.radio_value(&mut self.config.steam_exit, None, lang.text(s::NO_ACTION));
                    ui.radio_value(
                        &mut self.config.steam_exit,
                        Some(PowerCommand::Standby),
                        lang.text(s::STANDBY),
                    );
                    ui.radio_value(
                        &mut self.config.steam_exit,
                        Some(PowerCommand::Sleep),
                        lang.text(s::SLEEP),
                    );
                });
                ui.separator();
                ui.label(format!(
                    "{}: {}",
                    lang.text(s::DATA_DIRECTORY),
                    crate::config::directory().display()
                ));
                if ui.button(lang.text(s::SAVE)).clicked() {
                    self.persist();
                }
            });
        self.settings = open;
    }
    fn diagnosis(&mut self, ctx: &egui::Context) {
        let Some(id) = self.diagnosis.clone() else {
            return;
        };
        let Some(d) = self.devices.get(&id).cloned() else {
            return;
        };
        let lang = self.config.language;
        let mut open = true;
        let report = format!(
            "{}\nBluetooth: {}\n{}: {}\nID: {}\nRSSI ({}): {} / {}\n{}: {}\nStatus: {}\n{}: {}\n{}: {}\n\n{}",
            d.display_name(),
            d.known.name,
            lang.text(s::ADDRESS),
            d.known.address,
            id,
            lang.text(s::LAST_SCAN),
            d.rssi
                .map(|v| format!("{v} dBm"))
                .unwrap_or_else(|| "—".into()),
            d.seen.as_deref().unwrap_or("—"),
            lang.text(s::CONNECTED),
            if d.connected {
                lang.text(s::YES)
            } else {
                lang.text(s::NO)
            },
            d.status,
            lang.text(s::CHANNEL),
            d.channel
                .map(|v| v.to_string())
                .unwrap_or_else(|| "—".into()),
            lang.text(s::ERROR),
            d.error.as_deref().unwrap_or("—"),
            d.diagnosis
        );
        egui::Window::new(lang.text(s::DEVICE_DIAGNOSTICS))
            .open(&mut open)
            .default_size([760.0, 520.0])
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if ui
                        .add_enabled(!self.busy, egui::Button::new(lang.text(s::REFRESH)))
                        .clicked()
                    {
                        self.action(vec![id.clone()], Action::Diagnose);
                    }
                    if ui.button(lang.text(s::COPY_ALL)).clicked() {
                        ctx.copy_text(report.clone());
                    }
                    if ui.button(lang.text(s::EXPORT)).clicked() {
                        self.export("diagnosis", &report);
                    }
                });
                egui::ScrollArea::both()
                    .id_salt("diagnostic-scroll")
                    .show(ui, |ui| {
                        ui.add(
                            egui::Label::new(RichText::new(&report).monospace()).selectable(true),
                        );
                    });
            });
        if !open {
            self.diagnosis = None;
        }
    }
    fn confirmation(&mut self, ctx: &egui::Context) {
        let Some((ids, action)) = self.pending.clone() else {
            return;
        };
        let lang = self.config.language;
        egui::Window::new(lang.text(s::CONFIRM_DEVICE_ACTION))
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label(format!(
                    "{} — {} {}",
                    match action {
                        Action::Power(p) => lang
                            .text(match p {
                                PowerCommand::On => s::ON,
                                PowerCommand::Standby => s::STANDBY,
                                PowerCommand::Sleep => s::SLEEP,
                            })
                            .to_owned(),
                        Action::Channel(c) => format!("{} {}", lang.text(s::CHANNEL), c.byte()),
                        Action::Refresh => lang.text(s::REFRESH_STATUS).into(),
                        Action::Diagnose => lang.text(s::DIAGNOSTICS).into(),
                    },
                    ids.len(),
                    lang.text(s::STATIONS)
                ));
                for id in &ids {
                    if let Some(d) = self.devices.get(id) {
                        ui.label(format!("{} · {}", d.display_name(), d.known.address));
                    }
                }
                ui.horizontal(|ui| {
                    if ui
                        .add_enabled(!self.busy, egui::Button::new(lang.text(s::EXECUTE_NOW)))
                        .clicked()
                    {
                        self.pending = None;
                        self.send(Request::Run(ids, action));
                    }
                    if ui.button(lang.text(s::CANCEL)).clicked() {
                        self.pending = None;
                    }
                });
            });
    }
}
impl eframe::App for App {
    fn logic(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        self.events(ctx);
    }
    fn ui(&mut self, root: &mut egui::Ui, _: &mut eframe::Frame) {
        let ctx = root.ctx().clone();
        let lang = self.config.language;
        for theme in [egui::Theme::Dark, egui::Theme::Light] {
            let mut style = (*ctx.style_of(theme)).clone();
            style.spacing.interact_size.y = 28.0;
            style.spacing.button_padding.y = 5.0;
            ctx.set_style_of(theme, style);
        }
        ctx.set_theme(match self.config.theme {
            1 => egui::ThemePreference::Dark,
            2 => egui::ThemePreference::Light,
            _ => egui::ThemePreference::System,
        });
        egui::CentralPanel::default().show(root, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Basestation Tool");
                ui.label(RichText::new("LIGHTHOUSE / BLE").small().weak());
                if self.busy {
                    ui.spinner();
                    ui.label(lang.text(s::WORKING));
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let response = ui.button(format!("     {}", lang.text(s::SETTINGS)));
                    // Draw the icon ourselves so it cannot become a missing-font rectangle.
                    let center = egui::pos2(response.rect.left() + 13.0, response.rect.center().y);
                    let color = ui.visuals().text_color();
                    let points = (0..32)
                        .map(|i| {
                            let angle = i as f32 * std::f32::consts::TAU / 32.0;
                            let radius = if i % 4 == 0 || i % 4 == 3 { 7.0 } else { 9.0 };
                            center + egui::vec2(angle.cos(), angle.sin()) * radius
                        })
                        .collect();
                    ui.painter().add(egui::Shape::closed_line(
                        points,
                        egui::Stroke::new(1.3, color),
                    ));
                    ui.painter()
                        .circle_stroke(center, 3.0, egui::Stroke::new(1.3, color));
                    if response.clicked() {
                        self.settings = !self.settings;
                    }
                });
            });
            ui.horizontal_wrapped(|ui| {
                ui.add_enabled_ui(!self.busy && !self.no_ble, |ui| {
                    if ui.button(lang.text(s::SCAN)).clicked() {
                        self.last_scan = Instant::now();
                        self.send(Request::Scan);
                    }
                    if ui
                        .button(lang.text(s::RESET_BLUETOOTH))
                        .on_hover_text(lang.text(s::RESETS_ONLY_THIS_APPLICATION_S_CONNECTIONS))
                        .clicked()
                    {
                        self.send(Request::Reset);
                    }
                });
            });
            ui.separator();
            if self.no_ble {
                ui.colored_label(
                    ui.visuals().warn_fg_color,
                    lang.text(s::BLE_DISABLED_UI_TEST),
                );
            }
            if !self.save_allowed {
                ui.colored_label(
                    ui.visuals().warn_fg_color,
                    lang.text(s::INVALID_CONFIGURATION_ORIGINAL_RETAINED_SAVING_DISABLED),
                );
            }
            if !self.notice.is_empty() {
                ui.horizontal_wrapped(|ui| {
                    ui.label(&self.notice);
                    if ui.small_button("×").clicked() {
                        self.notice.clear();
                    }
                });
            }
            ui.horizontal(|ui| {
                if ui.button(lang.text(s::SELECT_ALL)).clicked() {
                    self.selected = self.devices.keys().cloned().collect();
                }
                if ui.button(lang.text(s::CLEAR_SELECTION)).clicked() {
                    self.selected.clear();
                }
                ui.label(format!(
                    "{}: {}/{}",
                    lang.text(s::SELECTED_LABEL),
                    self.selected.len(),
                    self.devices.len()
                ));
                ui.label(format!(
                    "SteamVR: {}",
                    lang.text(if self.steam_running {
                        s::RUNNING
                    } else {
                        s::STOPPED
                    })
                ));
            });
            let mut ids: Vec<_> = self.devices.keys().cloned().collect();
            ids.sort_by(|a, b| {
                let left = &self.devices[a];
                let right = &self.devices[b];
                let order = match self.sort_column {
                    0 => self.selected.contains(a).cmp(&self.selected.contains(b)),
                    1 => left
                        .display_name()
                        .to_lowercase()
                        .cmp(&right.display_name().to_lowercase()),
                    2 => left
                        .known
                        .name
                        .to_lowercase()
                        .cmp(&right.known.name.to_lowercase()),
                    3 => (left.online, left.busy, left.status.to_string()).cmp(&(
                        right.online,
                        right.busy,
                        right.status.to_string(),
                    )),
                    4 => left.channel.cmp(&right.channel),
                    5 => left.rssi.cmp(&right.rssi),
                    _ => left.known.address.cmp(&right.known.address),
                };
                let order = if self.sort_descending {
                    order.reverse()
                } else {
                    order
                };
                order.then_with(|| a.cmp(b))
            });
            let controls_height = if self.selected.len() == 1 {
                240.0
            } else {
                125.0
            };
            let max_height = (ui.available_height()
                - controls_height
                - if self.show_logs { 110.0 } else { 0.0 })
            .max(70.0);
            let height = if self.show_logs {
                self.table_height.clamp(70.0, max_height)
            } else {
                max_height
            };
            egui::ScrollArea::horizontal()
                .id_salt("device-horizontal-scroll")
                .show(ui, |ui| {
                    TableBuilder::new(ui)
                        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                        .striped(true)
                        .resizable(true)
                        .min_scrolled_height(height)
                        .max_scroll_height(height)
                        .auto_shrink([false, false])
                        .column(Column::initial(75.0).at_least(65.0))
                        .column(Column::initial(140.0))
                        .column(Column::initial(150.0))
                        .column(Column::initial(165.0))
                        .column(Column::initial(90.0).at_least(85.0))
                        .column(Column::initial(90.0))
                        .column(Column::initial(170.0))
                        .header(30.0, |mut h| {
                            for (column, label) in [
                                lang.text(s::SELECT),
                                lang.text(s::NAME),
                                "Bluetooth",
                                lang.text(s::STATE_CONNECTION),
                                lang.text(s::CHANNEL),
                                "RSSI",
                                lang.text(s::ADDRESS),
                            ]
                            .into_iter()
                            .enumerate()
                            {
                                h.col(|ui| {
                                    let suffix = if self.sort_column == column {
                                        if self.sort_descending { " v" } else { " ^" }
                                    } else {
                                        ""
                                    };
                                    if ui
                                        .selectable_label(
                                            self.sort_column == column,
                                            format!("{label}{suffix}"),
                                        )
                                        .on_hover_text(
                                            lang.text(
                                                s::CLICK_TO_SORT_CLICK_AGAIN_TO_REVERSE_ORDER,
                                            ),
                                        )
                                        .clicked()
                                    {
                                        if self.sort_column == column {
                                            self.sort_descending = !self.sort_descending;
                                        } else {
                                            self.sort_column = column;
                                            self.sort_descending = false;
                                        }
                                    }
                                });
                            }
                        })
                        .body(|body| {
                            body.rows(30.0, ids.len(), |mut row| {
                                let id = &ids[row.index()];
                                let d = self.devices.get_mut(id).unwrap();
                                row.set_selected(self.selected.contains(id));
                                row.col(|ui| {
                                    let mut checked = self.selected.contains(id);
                                    if ui.checkbox(&mut checked, "").changed() {
                                        if checked {
                                            self.selected.insert(id.clone());
                                        } else {
                                            self.selected.remove(id);
                                        }
                                    }
                                });
                                row.col(|ui| {
                                    if ui
                                        .selectable_label(
                                            self.selected.contains(id),
                                            d.display_name(),
                                        )
                                        .on_hover_text(lang.text(s::SELECT_ONLY_THIS_STATION))
                                        .clicked()
                                    {
                                        self.selected.clear();
                                        self.selected.insert(id.clone());
                                    }
                                });
                                row.col(|ui| {
                                    ui.label(&d.known.name);
                                });
                                row.col(|ui| {
                                    let text = if d.busy {
                                        lang.text(s::CHECKING).to_owned()
                                    } else if !d.online {
                                        lang.text(s::OFFLINE).into()
                                    } else {
                                        match d.status {
                                            Status::Unavailable => lang.text(s::UNAVAILABLE).into(),
                                            Status::On(b) => {
                                                format!("{} (0x{b:02X})", lang.text(s::ON_REPORTED))
                                            }
                                            Status::Unknown(b) => {
                                                format!("{} (0x{b:02X})", lang.text(s::UNKNOWN))
                                            }
                                            Status::Standby => lang.text(s::STANDBY).into(),
                                            Status::Sleep => lang.text(s::SLEEP).into(),
                                        }
                                    };
                                    let color = if d.error.is_some() {
                                        Color32::from_rgb(220, 135, 65)
                                    } else {
                                        ui.visuals().text_color()
                                    };
                                    ui.colored_label(color, text).on_hover_text(
                                        d.error.clone().unwrap_or_else(|| {
                                            format!(
                                                "{}: {}",
                                                lang.text(s::CONNECTED),
                                                lang.text(if d.connected { s::YES } else { s::NO })
                                            )
                                        }),
                                    );
                                });
                                row.col(|ui| {
                                    ui.label(
                                        d.channel
                                            .map(|v| v.to_string())
                                            .unwrap_or_else(|| "—".into()),
                                    );
                                });
                                row.col(|ui| {
                                    ui.label(
                                        d.rssi
                                            .map(|v| format!("{v} dBm"))
                                            .unwrap_or_else(|| "—".into()),
                                    )
                                    .on_hover_text(format!(
                                        "{}: {}",
                                        lang.text(s::LAST_SCAN_LABEL),
                                        d.seen.as_deref().unwrap_or("—")
                                    ));
                                });
                                row.col(|ui| {
                                    ui.label(&d.known.address).on_hover_text(id);
                                });
                            })
                        });
                });
            if self.show_logs {
                ui.add_space(12.0);
                let (rect, response) = ui.allocate_exact_size(
                    egui::vec2(ui.available_width(), 16.0),
                    egui::Sense::drag(),
                );
                let response = response
                    .on_hover_cursor(egui::CursorIcon::ResizeVertical)
                    .on_hover_text(lang.text(s::DRAG_TO_RESIZE_DEVICE_LIST_AND_LOG));
                let color = if response.hovered() || response.dragged() {
                    ui.visuals().text_color()
                } else {
                    ui.visuals().widgets.noninteractive.bg_stroke.color
                };
                ui.painter().hline(
                    rect.x_range(),
                    rect.center().y,
                    egui::Stroke::new(1.0, color),
                );
                ui.painter().hline(
                    (rect.center().x - 20.0)..=(rect.center().x + 20.0),
                    rect.center().y,
                    egui::Stroke::new(3.0, color),
                );
                if response.dragged() {
                    self.table_height =
                        (height + ui.input(|i| i.pointer.delta().y)).clamp(70.0, max_height);
                }
            }
            if self.devices.is_empty() {
                ui.label(lang.text(s::NO_STATIONS_FOUND_ENABLE_BLUETOOTH_AND_START_A_SCAN));
            }
            ui.separator();
            let targets = if self.selected.len() == 1 {
                device::targets(&self.devices, Scope::One(self.selected.first().unwrap()))
            } else {
                device::targets(&self.devices, Scope::Selected(&self.selected))
            };
            ui.group(|ui| {
                ui.set_min_width(ui.available_width());
                self.controls(ui, &targets);
                if let [single] = targets.as_slice() {
                    ui.separator();
                    let id = single.clone();
                    ui.horizontal_wrapped(|ui| {
                        if let Some(d) = self.devices.get_mut(&id) {
                            ui.label(lang.text(s::LOCAL_NAME));
                            ui.add(
                                egui::TextEdit::singleline(&mut d.known.alias)
                                    .desired_width(130.0)
                                    .hint_text(&d.known.name),
                            );
                        }
                        if ui.button(lang.text(s::SAVE)).clicked() {
                            self.persist();
                        }
                    });
                    ui.horizontal_wrapped(|ui| {
                        ui.label(lang.text(s::CHANNEL_LABEL));
                        egui::ComboBox::from_id_salt("channel")
                            .selected_text(self.channel.to_string())
                            .show_ui(ui, |ui| {
                                for n in 1..=16 {
                                    ui.selectable_value(&mut self.channel, n, n.to_string());
                                }
                            });
                        let supported = self.devices.get(&id).is_some_and(|d| d.supported);
                        if ui
                            .add_enabled(
                                !self.busy && supported && !self.no_ble,
                                egui::Button::new(lang.text(s::CHANGE_CHANNEL)),
                            )
                            .clicked()
                        {
                            self.action(
                                vec![id.clone()],
                                Action::Channel(Channel::new(self.channel).unwrap()),
                            );
                        }
                    });
                    ui.horizontal(|ui| {
                        if ui.button(lang.text(s::DIAGNOSTICS)).clicked() {
                            self.diagnosis = Some(id.clone());
                            if !self.busy {
                                self.action(vec![id], Action::Diagnose);
                            }
                        }
                    });
                }
            });
            if self.show_logs {
                ui.separator();
                ui.horizontal_wrapped(|ui| {
                    ui.strong(lang.text(s::LOG));
                    if ui.button(lang.text(s::CLEAR)).clicked() {
                        self.logs.entries.clear();
                    }
                    if ui.button(lang.text(s::SAVE_EXPORT)).clicked() {
                        self.export("log", &self.logs.full_text());
                    }
                    ui.add(
                        egui::TextEdit::singleline(&mut self.search)
                            .desired_width(170.0)
                            .hint_text(lang.text(s::SEARCH)),
                    );
                    egui::ComboBox::from_id_salt("level")
                        .selected_text(if self.level.is_empty() {
                            lang.text(s::ALL_LEVELS)
                        } else {
                            &self.level
                        })
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.level, String::new(), lang.text(s::ALL));
                            for l in ["Debug", "Info", "Warning", "Error"] {
                                ui.selectable_value(&mut self.level, l.into(), l);
                            }
                        });
                });
                if let Some(error) = &self.logs.disk_error {
                    ui.colored_label(ui.visuals().warn_fg_color, error);
                }
                egui::ScrollArea::vertical()
                    .id_salt("log-scroll")
                    .stick_to_bottom(true)
                    .auto_shrink([false, false])
                    .max_height(ui.available_height().max(30.0))
                    .show(ui, |ui| {
                        for e in &self.logs.entries {
                            if (self.level.is_empty() || self.level == e.level)
                                && (self.config.debug || e.level != "Debug")
                                && e.line.to_lowercase().contains(&self.search.to_lowercase())
                            {
                                ui.label(RichText::new(&e.line).monospace().size(12.0));
                            }
                        }
                    });
            }
        });
        self.settings(&ctx);
        self.diagnosis(&ctx);
        self.confirmation(&ctx);
    }
    fn save(&mut self, _: &mut dyn eframe::Storage) {
        self.persist();
    }
}
impl Drop for App {
    fn drop(&mut self) {
        self.persist();
        self.logs
            .push("Info", "Application shutdown", self.config.auto_log);
    }
}
