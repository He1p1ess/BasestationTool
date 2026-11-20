use eframe::egui::{self, Color32, Pos2, Stroke, Vec2};

#[derive(Clone, Copy)]
pub enum Icon {
    Scan,
    Refresh,
    Power,
    Moon,
    Sleep,
    Identify,
    Settings,
    Save,
    Clear,
}

fn icon(painter: &egui::Painter, center: Pos2, kind: Icon, color: Color32) {
    let stroke = Stroke::new(1.5, color);
    let point = |x, y| center + Vec2::new(x, y);
    let line = |a, b| {
        painter.line_segment([a, b], stroke);
    };
    match kind {
        Icon::Scan => {
            painter.add(egui::Shape::convex_polygon(
                vec![point(-4.0, -6.0), point(6.0, 0.0), point(-4.0, 6.0)],
                color,
                Stroke::NONE,
            ));
        }
        Icon::Power => {
            let points = (0..25)
                .map(|i| {
                    let angle = 0.65 + i as f32 * (std::f32::consts::TAU - 1.3) / 24.0;
                    point(angle.sin() * 6.0, -angle.cos() * 6.0)
                })
                .collect();
            painter.add(egui::Shape::line(points, stroke));
            line(point(0.0, -8.0), point(0.0, -1.0));
        }
        Icon::Moon => {
            painter.add(egui::Shape::closed_line(
                vec![
                    point(2.0, -7.0),
                    point(-3.0, -6.0),
                    point(-6.0, -2.0),
                    point(-5.0, 3.0),
                    point(-1.0, 6.0),
                    point(4.0, 5.0),
                    point(7.0, 1.0),
                    point(2.0, 2.0),
                    point(-1.0, -1.0),
                    point(-1.0, -4.0),
                ],
                stroke,
            ));
        }
        Icon::Sleep => {
            line(point(-6.0, -4.0), point(1.0, -4.0));
            line(point(1.0, -4.0), point(-6.0, 4.0));
            line(point(-6.0, 4.0), point(1.0, 4.0));
            line(point(3.0, -7.0), point(7.0, -7.0));
            line(point(7.0, -7.0), point(3.0, -2.0));
            line(point(3.0, -2.0), point(7.0, -2.0));
        }
        Icon::Identify => {
            painter.circle_stroke(center, 6.0, stroke);
            painter.circle_stroke(center, 2.5, stroke);
            for d in [Vec2::X, Vec2::Y, -Vec2::X, -Vec2::Y] {
                line(center + d * 5.0, center + d * 9.0);
            }
        }
        Icon::Refresh => {
            let points = (0..25)
                .map(|i| {
                    let angle = 0.6 + i as f32 * 5.0 / 24.0;
                    point(angle.cos() * 6.0, angle.sin() * 6.0)
                })
                .collect();
            painter.add(egui::Shape::line(points, stroke));
            line(point(6.0, -7.0), point(6.0, -2.0));
            line(point(6.0, -2.0), point(1.0, -2.0));
        }
        Icon::Settings => {
            let points = (0..32)
                .map(|i| {
                    let a = i as f32 * std::f32::consts::TAU / 32.0;
                    let r = if i % 4 < 2 { 7.0 } else { 9.0 };
                    point(a.cos() * r, a.sin() * r)
                })
                .collect();
            painter.add(egui::Shape::closed_line(points, stroke));
            painter.circle_stroke(center, 3.0, stroke);
        }
        Icon::Save => {
            painter.rect_stroke(
                egui::Rect::from_center_size(center, Vec2::splat(13.0)),
                2.0,
                stroke,
                egui::StrokeKind::Inside,
            );
            line(point(-3.0, -6.0), point(-3.0, -1.0));
            line(point(-3.0, -1.0), point(3.0, -1.0));
            line(point(3.0, -1.0), point(3.0, -6.0));
            line(point(-3.0, 3.0), point(3.0, 3.0));
        }
        Icon::Clear => {
            line(point(-5.0, -5.0), point(5.0, 5.0));
            line(point(-5.0, 5.0), point(5.0, -5.0));
        }
    }
}

pub fn button(ui: &mut egui::Ui, label: &str, kind: Icon, primary: bool) -> egui::Response {
    let text = format!("         {label}");
    let mut button = egui::Button::new(text);
    if primary {
        button = button.fill(ui.visuals().selection.bg_fill);
    }
    let response = ui.add(button);
    let color = if ui.is_enabled() {
        ui.visuals().text_color()
    } else {
        ui.visuals().weak_text_color()
    };
    icon(
        ui.painter(),
        egui::pos2(response.rect.left() + 17.0, response.rect.center().y),
        kind,
        color,
    );
    response
}

pub fn style(ctx: &egui::Context) {
    for theme in [egui::Theme::Dark, egui::Theme::Light] {
        let mut style = (*ctx.style_of(theme)).clone();
        let dark = theme == egui::Theme::Dark;
        let (bg, card, button, hover, ink, muted, border, accent, selected) = if dark {
            (
                0x201d27, 0x292431, 0x37303f, 0x493650, 0xf7f0fa, 0xc4b7ce, 0x514457, 0xd6a1ef,
                0x693c7b,
            )
        } else {
            (
                0xfff9ff, 0xf9effa, 0xf4e6f6, 0xebd5ee, 0x2b2031, 0x68566e, 0xddc9e2, 0x8b348f,
                0xe7c6ef,
            )
        };
        let color = |hex: u32| Color32::from_rgb((hex >> 16) as u8, (hex >> 8) as u8, hex as u8);
        let v = &mut style.visuals;
        v.override_text_color = Some(color(ink));
        v.weak_text_color = Some(color(muted));
        v.panel_fill = color(bg);
        v.window_fill = color(card);
        v.extreme_bg_color = color(bg);
        v.faint_bg_color = color(card);
        v.code_bg_color = color(card);
        v.hyperlink_color = color(accent);
        v.selection.bg_fill = color(selected);
        v.selection.stroke = Stroke::new(1.0, color(ink));
        v.window_stroke = Stroke::new(1.0, color(border));
        v.window_corner_radius = 10.into();
        for widget in [
            &mut v.widgets.noninteractive,
            &mut v.widgets.inactive,
            &mut v.widgets.hovered,
            &mut v.widgets.active,
            &mut v.widgets.open,
        ] {
            widget.corner_radius = 7.into();
            widget.bg_stroke = Stroke::new(1.0, color(border));
            widget.fg_stroke = Stroke::new(1.5, color(ink));
            widget.bg_fill = color(button);
            widget.weak_bg_fill = color(button);
        }
        v.widgets.noninteractive.bg_fill = color(card);
        v.widgets.noninteractive.weak_bg_fill = color(card);
        v.widgets.hovered.bg_fill = color(hover);
        v.widgets.hovered.weak_bg_fill = color(hover);
        v.widgets.active.bg_fill = color(selected);
        v.widgets.active.weak_bg_fill = color(selected);
        style.spacing.interact_size.y = 34.0;
        style.spacing.window_margin = 16.into();
        style.spacing.button_padding = egui::vec2(12.0, 7.0);
        style.spacing.item_spacing = egui::vec2(10.0, 10.0);
        style
            .text_styles
            .insert(egui::TextStyle::Heading, egui::FontId::proportional(26.0));
        ctx.set_style_of(theme, style);
    }
}

pub fn title_bar(root: &mut egui::Ui) {
    let ctx = root.ctx().clone();
    egui::Panel::top("custom-title-bar")
        .exact_size(44.0)
        .frame(
            egui::Frame::new()
                .fill(root.visuals().window_fill)
                .inner_margin(egui::Margin::symmetric(12, 5)),
        )
        .show(root, |ui| {
            let width = ui.available_width();
            ui.horizontal(|ui| {
                let (rect, response) = ui.allocate_exact_size(
                    egui::vec2((width - 130.0).max(40.0), 32.0),
                    egui::Sense::click_and_drag(),
                );
                let center = egui::pos2(rect.left() + 12.0, rect.center().y);
                ui.painter().rect_filled(
                    egui::Rect::from_center_size(center, Vec2::splat(26.0)),
                    6.0,
                    ui.visuals().selection.bg_fill,
                );
                let c = ui.visuals().text_color();
                let pts = [
                    (-7.0, -4.0),
                    (0.0, -8.0),
                    (7.0, -4.0),
                    (7.0, 4.0),
                    (0.0, 8.0),
                    (-7.0, 4.0),
                ];
                ui.painter().add(egui::Shape::closed_line(
                    pts.map(|(x, y)| center + egui::vec2(x, y)).to_vec(),
                    Stroke::new(1.2, c),
                ));
                for (a, b) in [
                    ((-7.0, -4.0), (0.0, 0.0)),
                    ((7.0, -4.0), (0.0, 0.0)),
                    ((0.0, 0.0), (0.0, 8.0)),
                ] {
                    ui.painter().line_segment(
                        [center + egui::vec2(a.0, a.1), center + egui::vec2(b.0, b.1)],
                        Stroke::new(1.2, c),
                    );
                }
                ui.painter().text(
                    rect.left_center() + egui::vec2(35.0, 0.0),
                    egui::Align2::LEFT_CENTER,
                    "Basestation Tool",
                    egui::FontId::proportional(16.0),
                    c,
                );
                if response.double_clicked() {
                    let max = ctx.input(|i| i.viewport().maximized.unwrap_or(false));
                    ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(!max));
                } else if response.drag_started() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                }
                for action in 0..3 {
                    let (r, res) =
                        ui.allocate_exact_size(egui::vec2(32.0, 32.0), egui::Sense::click());
                    if res.hovered() {
                        ui.painter()
                            .rect_filled(r, 5.0, ui.visuals().widgets.hovered.bg_fill);
                    }
                    let p = r.center();
                    let stroke = Stroke::new(1.4, c);
                    match action {
                        0 => {
                            ui.painter().line_segment(
                                [p + egui::vec2(-6.0, 0.0), p + egui::vec2(6.0, 0.0)],
                                stroke,
                            );
                        }
                        1 => {
                            ui.painter().rect_stroke(
                                egui::Rect::from_center_size(p, Vec2::splat(12.0)),
                                1.0,
                                stroke,
                                egui::StrokeKind::Inside,
                            );
                        }
                        _ => {
                            icon(ui.painter(), p, Icon::Clear, c);
                        }
                    }
                    let res =
                        res.on_hover_text(["Minimize", "Maximize / Restore", "Close"][action]);
                    if res.clicked() {
                        ctx.send_viewport_cmd(match action {
                            0 => egui::ViewportCommand::Minimized(true),
                            1 => egui::ViewportCommand::Maximized(
                                !ctx.input(|i| i.viewport().maximized.unwrap_or(false)),
                            ),
                            _ => egui::ViewportCommand::Close,
                        });
                    }
                }
            });
        });
}

pub fn resize_edges(root: &egui::Ui) {
    if root.input(|i| i.viewport().maximized.unwrap_or(false)) {
        return;
    }
    let Some(pos) = root.input(|i| i.pointer.hover_pos()) else {
        return;
    };
    let rect = root.max_rect();
    let left = pos.x < rect.left() + 5.0;
    let right = pos.x > rect.right() - 5.0;
    let top = pos.y < rect.top() + 5.0;
    let bottom = pos.y > rect.bottom() - 5.0;
    use egui::{CursorIcon as C, ResizeDirection as R};
    let (direction, cursor) = match (left, right, top, bottom) {
        (true, _, true, _) => (R::NorthWest, C::ResizeNwSe),
        (_, true, true, _) => (R::NorthEast, C::ResizeNeSw),
        (true, _, _, true) => (R::SouthWest, C::ResizeNeSw),
        (_, true, _, true) => (R::SouthEast, C::ResizeNwSe),
        (true, _, _, _) => (R::West, C::ResizeHorizontal),
        (_, true, _, _) => (R::East, C::ResizeHorizontal),
        (_, _, true, _) => (R::North, C::ResizeVertical),
        (_, _, _, true) => (R::South, C::ResizeVertical),
        _ => return,
    };
    root.ctx().set_cursor_icon(cursor);
    if root.input(|i| i.pointer.primary_pressed()) {
        root.ctx()
            .send_viewport_cmd(egui::ViewportCommand::BeginResize(direction));
    }
}
