use super::EmberFatParams;
use nih_plug::prelude::*;
use nih_plug_egui::{create_egui_editor, egui};
use std::sync::Arc;

const PANEL_SIZE: egui::Vec2 = egui::vec2(1000.0, 540.0);
const CREAM: egui::Color32 = egui::Color32::from_rgb(249, 239, 222);
const INK: egui::Color32 = egui::Color32::from_rgb(22, 23, 25);
const ACCENT: egui::Color32 = egui::Color32::from_rgb(255, 211, 151);

pub(super) fn create(params: Arc<EmberFatParams>) -> Option<Box<dyn Editor>> {
    create_egui_editor(
        params.editor_state.clone(),
        Option::<Artwork>::None,
        |ctx, artwork| {
            configure_style(ctx);
            *artwork = Some(load_artwork(ctx));
        },
        move |ctx, setter, artwork| {
            egui::CentralPanel::default()
                .frame(egui::Frame::new().fill(INK).inner_margin(0.0))
                .show(ctx, |ui| {
                    if let Some(artwork) = artwork {
                        draw_front_panel(ui, artwork, &params, setter);
                    }
                });
        },
    )
}

fn configure_style(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "barlow".into(),
        egui::FontData::from_static(include_bytes!(
            "../assets/fonts/BarlowCondensed-SemiBold.ttf"
        ))
        .into(),
    );
    fonts.families.insert(
        egui::FontFamily::Name("display".into()),
        vec!["barlow".into()],
    );
    ctx.set_fonts(fonts);
    let mut style = (*ctx.style()).clone();
    style.visuals = egui::Visuals::dark();
    style.visuals.panel_fill = INK;
    style.visuals.selection.bg_fill = egui::Color32::from_rgb(93, 76, 58);
    style.visuals.selection.stroke.color = CREAM;
    ctx.set_style(style);
}

fn display_font(size: f32) -> egui::FontId {
    egui::FontId::new(size, egui::FontFamily::Name("display".into()))
}

#[derive(Clone, Copy)]
struct PanelLayout {
    origin: egui::Pos2,
    scale: f32,
}

impl PanelLayout {
    fn new(rect: egui::Rect) -> Self {
        let scale = (rect.width() / PANEL_SIZE.x).min(rect.height() / PANEL_SIZE.y);
        Self {
            origin: rect.center() - PANEL_SIZE * scale * 0.5,
            scale,
        }
    }

    fn point(self, x: f32, y: f32) -> egui::Pos2 {
        self.origin + egui::vec2(x, y) * self.scale
    }

    fn rect(self, x: f32, y: f32, width: f32, height: f32) -> egui::Rect {
        egui::Rect::from_min_size(self.point(x, y), egui::vec2(width, height) * self.scale)
    }
}

struct Artwork {
    background: egui::TextureHandle,
    knob: egui::TextureHandle,
    knob_range: egui::TextureHandle,
    mascot_stages: [egui::TextureHandle; 6],
}

fn load_artwork(ctx: &egui::Context) -> Artwork {
    let background = load_rgba_texture(
        ctx,
        "emberfat-metal-panel",
        [900, 486],
        include_bytes!("../assets/metal-background-900x486.rgba"),
    );
    let knob = load_rgba_texture(
        ctx,
        "emberfat-silver-knob",
        [384, 384],
        include_bytes!("../assets/silver-knob-384.rgba"),
    );
    let knob_range = load_neutral_texture(
        ctx,
        "emberfat-knob-range",
        [256, 256],
        include_bytes!("../assets/knob-range-256.rgba"),
    );
    let mascot_stages = [
        load_rgba_texture(
            ctx,
            "fat-stage-1",
            [240, 390],
            include_bytes!("../assets/fat-stage-1.rgba"),
        ),
        load_rgba_texture(
            ctx,
            "fat-stage-2",
            [240, 390],
            include_bytes!("../assets/fat-stage-2.rgba"),
        ),
        load_rgba_texture(
            ctx,
            "fat-stage-3",
            [240, 390],
            include_bytes!("../assets/fat-stage-3.rgba"),
        ),
        load_rgba_texture(
            ctx,
            "fat-stage-4",
            [240, 390],
            include_bytes!("../assets/fat-stage-4.rgba"),
        ),
        load_rgba_texture(
            ctx,
            "fat-stage-5",
            [240, 390],
            include_bytes!("../assets/fat-stage-5.rgba"),
        ),
        load_rgba_texture(
            ctx,
            "fat-stage-6",
            [240, 390],
            include_bytes!("../assets/fat-stage-6.rgba"),
        ),
    ];
    Artwork {
        background,
        knob,
        knob_range,
        mascot_stages,
    }
}

fn load_rgba_texture(
    ctx: &egui::Context,
    name: &str,
    size: [usize; 2],
    rgba: &[u8],
) -> egui::TextureHandle {
    debug_assert_eq!(rgba.len(), size[0] * size[1] * 4);
    let image = egui::ColorImage::from_rgba_unmultiplied(size, rgba);
    ctx.load_texture(name, image, egui::TextureOptions::LINEAR)
}

fn load_neutral_texture(
    ctx: &egui::Context,
    name: &str,
    size: [usize; 2],
    rgba: &[u8],
) -> egui::TextureHandle {
    let mut neutral = rgba.to_vec();
    for pixel in neutral.chunks_exact_mut(4) {
        let gray = ((u16::from(pixel[0]) + u16::from(pixel[1]) + u16::from(pixel[2])) / 3) as u8;
        pixel[..3].fill(gray);
    }
    load_rgba_texture(ctx, name, size, &neutral)
}

fn draw_front_panel(
    ui: &mut egui::Ui,
    artwork: &Artwork,
    params: &EmberFatParams,
    setter: &ParamSetter,
) {
    let layout = PanelLayout::new(ui.max_rect());
    let painter = ui.painter();
    // Sample the panel itself, excluding the transparent margins in the supplied artwork.
    painter.image(
        artwork.background.id(),
        layout.rect(0.0, 0.0, 1000.0, 540.0),
        egui::Rect::from_min_max(egui::pos2(0.018, 0.100), egui::pos2(0.985, 0.905)),
        egui::Color32::WHITE,
    );
    panel_text(
        painter,
        layout.point(74.0, 76.0),
        egui::Align2::LEFT_CENTER,
        "EMBERFAT",
        display_font(35.0 * layout.scale),
        CREAM,
    );
    panel_text(
        painter,
        layout.point(75.0, 104.0),
        egui::Align2::LEFT_CENTER,
        "HARMONIC SATURATION",
        egui::FontId::proportional(10.0 * layout.scale),
        CREAM,
    );
    ui.interact(
        layout.rect(70.0, 54.0, 205.0, 62.0),
        egui::Id::new("brand"),
        egui::Sense::hover(),
    )
    .on_hover_text(format!(
        "EmberFat {}\nStereo saturation · Ember Audio Lab",
        env!("CARGO_PKG_VERSION")
    ));

    let stage = ((params.fat.value() / 100.0).clamp(0.0, 1.0) * 5.0).round() as usize;
    ui.painter().image(
        artwork.mascot_stages[stage].id(),
        layout.rect(64.0, 136.0, 220.0, 357.5),
        egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)),
        egui::Color32::WHITE,
    );

    draw_dial(
        ui,
        artwork,
        layout,
        (438.0, 206.0),
        86.0,
        "FAT",
        "Increases drive and saturation density.",
        &params.fat,
        setter,
        false,
    );
    draw_dial(
        ui,
        artwork,
        layout,
        (700.0, 238.0),
        73.0,
        "COLOR",
        "Changes the saturation character.",
        &params.color,
        setter,
        false,
    );
    draw_dial(
        ui,
        artwork,
        layout,
        (530.0, 420.0),
        35.0,
        "MIX",
        "Blends dry and processed sound.",
        &params.mix,
        setter,
        false,
    );
    draw_dial(
        ui,
        artwork,
        layout,
        (764.0, 448.0),
        19.0,
        "OUTPUT",
        "Sets the processed signal level before the dry/wet blend.",
        &params.output,
        setter,
        true,
    );

    draw_bypass(ui, layout, &params.bypass, setter);
}

fn panel_text(
    painter: &egui::Painter,
    position: egui::Pos2,
    align: egui::Align2,
    text: &str,
    font: egui::FontId,
    color: egui::Color32,
) {
    painter.text(
        position + egui::vec2(0.7, 1.0),
        align,
        text,
        font.clone(),
        egui::Color32::from_black_alpha(230),
    );
    painter.text(position, align, text, font, color);
}

#[allow(clippy::too_many_arguments)]
fn draw_dial(
    ui: &mut egui::Ui,
    artwork: &Artwork,
    layout: PanelLayout,
    position: (f32, f32),
    radius: f32,
    label: &'static str,
    description: &'static str,
    param: &FloatParam,
    setter: &ParamSetter,
    is_output: bool,
) {
    let center = layout.point(position.0, position.1);
    let size = ((radius * 2.0 + 24.0) * layout.scale).max(44.0);
    let mut response = ui.interact(
        egui::Rect::from_center_size(center, egui::vec2(size, size)),
        egui::Id::new(("emberfat-dial", label)),
        egui::Sense::click_and_drag(),
    );
    let value = interact_float_param(ui, &mut response, param, setter, layout.scale);
    response.widget_info(|| egui::WidgetInfo::slider(ui.is_enabled(), value as f64, label));
    let active = response.hovered() || response.dragged() || response.has_focus();
    let normalized = param.preview_normalized(value);
    draw_rotary(
        ui.painter(),
        artwork,
        center,
        radius * layout.scale,
        normalized,
        is_output,
        active,
    );

    let compact = radius < 40.0;
    let label_offset = if is_output { 31.0 } else { radius + 27.0 };
    let value_offset = label_offset + if compact { 19.0 } else { 23.0 };
    panel_text(
        ui.painter(),
        center + egui::vec2(0.0, label_offset * layout.scale),
        egui::Align2::CENTER_CENTER,
        label,
        display_font(if compact { 17.0 } else { 25.0 } * layout.scale),
        if active { ACCENT } else { CREAM },
    );

    let value_rect = egui::Rect::from_center_size(
        center + egui::vec2(0.0, value_offset * layout.scale),
        egui::vec2(if is_output { 98.0 } else { 82.0 }, 24.0) * layout.scale,
    );
    draw_value(
        ui,
        value_rect,
        value,
        param,
        setter,
        is_output,
        layout.scale,
        label,
    );
    response.on_hover_cursor(egui::CursorIcon::ResizeVertical).on_hover_text(format!(
        "{description}\nDrag up/down · Shift for fine control\nDouble-click to reset · Click the value to type\nArrow keys adjust a focused knob."
    ));
}

fn draw_value(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    value: f32,
    param: &FloatParam,
    setter: &ParamSetter,
    is_output: bool,
    scale: f32,
    label: &str,
) {
    let mut edited_value = value;
    let (min, max) = parameter_bounds(param);
    let response = ui
        .push_id(("value", label), |ui| {
            ui.scope(|ui| {
                let style = ui.style_mut();
                style.override_font_id = Some(egui::FontId::proportional(13.0 * scale));
                style.spacing.interact_size = rect.size();
                style.spacing.button_padding = egui::vec2(3.0, 2.0) * scale;
                for widget in [
                    &mut style.visuals.widgets.inactive,
                    &mut style.visuals.widgets.hovered,
                    &mut style.visuals.widgets.active,
                    &mut style.visuals.widgets.open,
                ] {
                    widget.bg_fill = egui::Color32::TRANSPARENT;
                    widget.weak_bg_fill = egui::Color32::TRANSPARENT;
                    widget.bg_stroke = egui::Stroke::NONE;
                    widget.fg_stroke.color = CREAM;
                }
                ui.put(
                    rect,
                    egui::DragValue::new(&mut edited_value)
                        .range(min..=max)
                        .speed(0.1)
                        .fixed_decimals(1)
                        .suffix(if is_output { " dB" } else { "%" })
                        .custom_parser(|text| {
                            text.trim()
                                .trim_end_matches("dB")
                                .trim_end_matches('%')
                                .trim()
                                .replace(',', ".")
                                .parse::<f64>()
                                .ok()
                                .filter(|value| value.is_finite())
                        })
                        .update_while_editing(false),
                )
            })
            .inner
        })
        .inner;
    if response.drag_started() {
        setter.begin_set_parameter(param);
    }
    if edited_value != value && edited_value.is_finite() {
        if !response.dragged() && !response.drag_stopped() {
            setter.begin_set_parameter(param);
        }
        setter.set_parameter(param, edited_value.clamp(min, max));
        if !response.dragged() && !response.drag_stopped() {
            setter.end_set_parameter(param);
        }
    }
    if response.drag_stopped() {
        setter.end_set_parameter(param);
    }
    response.on_hover_text(
        "Click to type a value. Enter applies; Escape cancels. Double-click the knob to reset.",
    );
}

fn draw_rotary(
    painter: &egui::Painter,
    artwork: &Artwork,
    center: egui::Pos2,
    radius: f32,
    normalized: f32,
    is_output: bool,
    active: bool,
) {
    if is_output {
        // A small, exactly circular gain cap with fixed lighting.
        painter.circle_filled(center, radius, egui::Color32::from_rgb(13, 14, 16));
        painter.circle_filled(
            center,
            radius * 0.93,
            egui::Color32::from_rgb(174, 177, 178),
        );
        painter.circle_filled(
            center + egui::vec2(0.0, radius * 0.04),
            radius * 0.83,
            egui::Color32::from_rgb(47, 49, 51),
        );
        painter.circle_filled(
            center - egui::vec2(0.0, radius * 0.10),
            radius * 0.68,
            egui::Color32::from_rgb(62, 65, 68),
        );
    } else {
        let range_size = radius * 2.28;
        painter.image(
            artwork.knob_range.id(),
            egui::Rect::from_center_size(center, egui::vec2(range_size, range_size)),
            egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)),
            if active {
                egui::Color32::WHITE
            } else {
                egui::Color32::from_gray(215)
            },
        );
        // The housing and metal highlights never rotate; this prevents perspective wobble.
        painter.image(
            artwork.knob.id(),
            egui::Rect::from_center_size(center, egui::vec2(radius * 2.0, radius * 2.0)),
            egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)),
            egui::Color32::WHITE,
        );
    }
    let angle = std::f32::consts::PI * (0.75 + normalized.clamp(0.0, 1.0) * 1.5);
    let direction = egui::vec2(angle.cos(), angle.sin());
    let endpoints = [
        center + direction * radius * 0.32,
        center + direction * radius * 0.67,
    ];
    let width = (radius * 0.10).max(2.0);
    capsule(
        painter,
        [
            endpoints[0] + egui::vec2(0.6, 0.9),
            endpoints[1] + egui::vec2(0.6, 0.9),
        ],
        width + 2.5,
        egui::Color32::from_black_alpha(180),
    );
    capsule(
        painter,
        endpoints,
        width + 1.0,
        egui::Color32::from_rgb(241, 234, 220),
    );
    capsule(
        painter,
        endpoints,
        width * 0.57,
        egui::Color32::from_rgb(255, 251, 240),
    );
}

fn capsule(painter: &egui::Painter, points: [egui::Pos2; 2], width: f32, color: egui::Color32) {
    painter.line_segment(points, egui::Stroke::new(width, color));
    for point in points {
        painter.circle_filled(point, width * 0.5, color);
    }
}

#[derive(Clone, Copy)]
struct PendingBypass {
    value: bool,
    requested_at: f64,
}

fn draw_bypass(ui: &mut egui::Ui, layout: PanelLayout, param: &BoolParam, setter: &ParamSetter) {
    let rect = layout.rect(312.0, 439.0, 128.0, 35.0);
    let response = ui.interact(rect, egui::Id::new("emberfat-bypass"), egui::Sense::click());
    let pending_id = response.id.with("pending");
    let now = ui.input(|input| input.time);
    let pending = ui.data(|data| data.get_temp::<PendingBypass>(pending_id));
    let mut bypassed = match pending {
        Some(pending) if now - pending.requested_at < 0.5 && pending.value != param.value() => {
            pending.value
        }
        _ => {
            ui.data_mut(|data| data.remove::<PendingBypass>(pending_id));
            param.value()
        }
    };
    if response.clicked() {
        bypassed = !bypassed;
        ui.data_mut(|data| {
            data.insert_temp(
                pending_id,
                PendingBypass {
                    value: bypassed,
                    requested_at: now,
                },
            )
        });
        setter.begin_set_parameter(param);
        setter.set_parameter(param, bypassed);
        setter.end_set_parameter(param);
    }
    response.widget_info(|| {
        egui::WidgetInfo::selected(
            egui::WidgetType::Checkbox,
            ui.is_enabled(),
            bypassed,
            "Bypass",
        )
    });
    let painter = ui.painter();
    painter.rect_filled(
        rect.translate(egui::vec2(0.0, 2.0 * layout.scale)),
        17.0 * layout.scale,
        egui::Color32::from_black_alpha(180),
    );
    painter.rect_filled(
        rect,
        17.0 * layout.scale,
        if response.hovered() || response.has_focus() {
            egui::Color32::from_rgb(53, 50, 47)
        } else {
            INK
        },
    );
    let led = layout.point(328.0, 456.5);
    painter.circle_filled(
        led,
        4.0 * layout.scale,
        if bypassed {
            egui::Color32::from_rgb(100, 99, 96)
        } else {
            egui::Color32::from_rgb(242, 126, 71)
        },
    );
    painter.text(
        layout.point(384.0, 456.5),
        egui::Align2::CENTER_CENTER,
        if bypassed { "BYPASSED" } else { "ACTIVE" },
        display_font(15.0 * layout.scale),
        CREAM,
    );
    response
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .on_hover_text(
            "Compare with the original signal. ACTIVE = processing; BYPASSED = dry signal.",
        );
}

fn parameter_bounds(param: &FloatParam) -> (f32, f32) {
    (param.preview_plain(0.0), param.preview_plain(1.0))
}

fn set_once(param: &FloatParam, setter: &ParamSetter, value: f32) {
    if value.is_finite() {
        let (min, max) = parameter_bounds(param);
        setter.begin_set_parameter(param);
        setter.set_parameter(param, value.clamp(min, max));
        setter.end_set_parameter(param);
    }
}

#[derive(Clone, Copy)]
struct DragState {
    value: f32,
}

fn interact_float_param(
    ui: &egui::Ui,
    response: &mut egui::Response,
    param: &FloatParam,
    setter: &ParamSetter,
    scale: f32,
) -> f32 {
    let (min, max) = parameter_bounds(param);
    let drag_id = response.id.with("precise-drag");
    let mut value = param.value();
    if response.drag_started() {
        response.request_focus();
        ui.data_mut(|data| data.insert_temp(drag_id, DragState { value }));
        setter.begin_set_parameter(param);
    }
    if response.dragged() {
        let mut drag = ui
            .data(|data| data.get_temp::<DragState>(drag_id))
            .unwrap_or(DragState { value });
        let fine = ui.input(|input| input.modifiers.shift);
        let multiplier = if fine { 0.1 } else { 1.0 };
        // Accumulate independently of the host's delayed/quantized parameter feedback.
        drag.value = (drag.value
            - response.drag_motion().y / scale.max(0.1) * (max - min) / 160.0 * multiplier)
            .clamp(min, max);
        value = drag.value;
        ui.data_mut(|data| data.insert_temp(drag_id, drag));
        setter.set_parameter(param, value);
        response.mark_changed();
    }
    if response.drag_stopped() {
        setter.end_set_parameter(param);
        ui.data_mut(|data| data.remove::<DragState>(drag_id));
    }
    if response.double_clicked() {
        response.request_focus();
        value = param.default_plain_value();
        set_once(param, setter, value);
        response.mark_changed();
    } else if response.clicked() {
        response.request_focus();
    }

    if !response.dragged() && !response.drag_stopped() {
        let fine = ui.input(|input| input.modifiers.shift);
        let step = if fine { 0.1 } else { 1.0 };
        let mut change = 0.0;
        if response.has_focus() {
            ui.input_mut(|input| {
                if input.consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp)
                    || input.consume_key(egui::Modifiers::SHIFT, egui::Key::ArrowUp)
                    || input.consume_key(egui::Modifiers::NONE, egui::Key::ArrowRight)
                    || input.consume_key(egui::Modifiers::SHIFT, egui::Key::ArrowRight)
                {
                    change += step;
                }
                if input.consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown)
                    || input.consume_key(egui::Modifiers::SHIFT, egui::Key::ArrowDown)
                    || input.consume_key(egui::Modifiers::NONE, egui::Key::ArrowLeft)
                    || input.consume_key(egui::Modifiers::SHIFT, egui::Key::ArrowLeft)
                {
                    change -= step;
                }
            });
        }
        if response.hovered() && (!ui.ctx().wants_keyboard_input() || response.has_focus()) {
            ui.input_mut(|input| {
                let wheel = input.raw_scroll_delta.y;
                if wheel != 0.0 {
                    change += (wheel / 40.0).clamp(-10.0, 10.0) * step;
                    input.raw_scroll_delta = egui::Vec2::ZERO;
                    input.smooth_scroll_delta = egui::Vec2::ZERO;
                }
            });
        }
        if change != 0.0 {
            value = (value + change).clamp(min, max);
            set_once(param, setter, value);
            response.mark_changed();
        }
    }
    value
}

#[cfg(test)]
mod tests;
