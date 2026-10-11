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
    controls: egui::TextureHandle,
    switches: egui::TextureHandle,
    tigers: [egui::TextureHandle; 2],
}

fn load_artwork(ctx: &egui::Context) -> Artwork {
    Artwork {
        background: load_rgba_texture(
            ctx,
            "emberfat-amber-panel",
            [1709, 920],
            include_bytes!("../assets/amber-panel.rgba"),
        ),
        controls: load_rgba_texture(
            ctx,
            "emberfat-supplied-controls",
            [2048, 768],
            include_bytes!("../assets/control-sheet.rgba"),
        ),
        switches: load_rgba_texture(
            ctx,
            "emberfat-active-bypass",
            [2048, 768],
            include_bytes!("../assets/switch-sheet.rgba"),
        ),
        tigers: [
            load_rgba_texture(
                ctx,
                "emberfat-tigers-left",
                [1008, 724],
                include_bytes!("../assets/tigers-left.rgba"),
            ),
            load_rgba_texture(
                ctx,
                "emberfat-tigers-right",
                [1164, 724],
                include_bytes!("../assets/tigers-right.rgba"),
            ),
        ],
    }
}

// Sprite rectangles reference the original supplied sheets; no resampling or
// recoloring is baked into the source artwork. Pivots align rotating caps and rings.
fn sprite(
    painter: &egui::Painter,
    texture: &egui::TextureHandle,
    source: [f32; 4],
    pivot: egui::Pos2,
    center: egui::Pos2,
    scale: f32,
    rotation: f32,
) {
    let [left, top, right, bottom] = source;
    let tex_size = texture.size_vec2();
    let turn = egui::emath::Rot2::from_angle(rotation);
    let mut mesh = egui::Mesh::with_texture(texture.id());
    for point in [
        egui::pos2(left, top),
        egui::pos2(right, top),
        egui::pos2(right, bottom),
        egui::pos2(left, bottom),
    ] {
        mesh.vertices.push(egui::epaint::Vertex {
            pos: center + turn * ((point - pivot) * scale),
            uv: egui::pos2(point.x / tex_size.x, point.y / tex_size.y),
            color: egui::Color32::WHITE,
        });
    }
    mesh.indices.extend_from_slice(&[0, 1, 2, 0, 2, 3]);
    painter.add(egui::Shape::mesh(mesh));
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

fn draw_front_panel(
    ui: &mut egui::Ui,
    artwork: &Artwork,
    params: &EmberFatParams,
    setter: &ParamSetter,
) {
    let layout = PanelLayout::new(ui.max_rect());
    let painter = ui.painter();
    // The background texture already matches the editor's aspect ratio and has no transparent margins.
    painter.image(
        artwork.background.id(),
        layout.rect(0.0, 0.0, 1000.0, 540.0),
        egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)),
        egui::Color32::WHITE,
    );
    // The supplied faceplate includes the EMBERFAT wordmark.
    ui.interact(
        layout.rect(28.0, 20.0, 380.0, 125.0),
        egui::Id::new("brand"),
        egui::Sense::hover(),
    )
    .on_hover_text(format!(
        "EmberFat {}\nStereo saturation · eM4",
        env!("CARGO_PKG_VERSION")
    ));

    let stage = ((params.fat.value() / 100.0).clamp(0.0, 1.0) * 5.0).round() as usize;
    let mut bounds = [
        [12.0, 80.0, 314.0, 668.0],
        [318.0, 80.0, 652.0, 668.0],
        [657.0, 80.0, 1006.0, 668.0],
        [1009.0, 80.0, 1377.0, 668.0],
        [1378.0, 80.0, 1755.0, 668.0],
        [1756.0, 80.0, 2160.0, 668.0],
    ][stage];
    let tiger_sheet = if stage < 3 { 0 } else { 1 };
    if tiger_sheet == 1 {
        bounds[0] -= 1008.0;
        bounds[2] -= 1008.0;
    }
    sprite(
        painter,
        &artwork.tigers[tiger_sheet],
        bounds,
        egui::pos2(
            (bounds[0] + bounds[2]) * 0.5,
            [646.0, 648.0, 650.0, 651.0, 653.0, 654.0][stage],
        ),
        layout.point(165.0, 485.0),
        0.58 * layout.scale,
        0.0,
    );

    draw_dial(
        ui,
        artwork,
        layout,
        (445.0, 206.0),
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
        (702.0, 238.0),
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
        (538.0, 413.0),
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
        (800.0, 436.0),
        19.0,
        "OUTPUT",
        "Sets the processed signal level before the dry/wet blend.",
        &params.output,
        setter,
        true,
    );

    draw_bypass(ui, artwork, layout, &params.bypass, setter);
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
        if color == INK {
            egui::Color32::from_white_alpha(145)
        } else {
            egui::Color32::from_black_alpha(230)
        },
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
        if is_output {
            3
        } else if radius < 40.0 {
            2
        } else if radius < 80.0 {
            1
        } else {
            0
        },
        active,
    );

    let compact = radius < 40.0;
    // Label/value groups are anchored to the faceplate's white paint patches,
    // independently of the cap size and the static range artwork.
    let text_center = match label {
        "FAT" => (440.0, 312.0),
        "COLOR" => (709.0, 329.0),
        "MIX" => (537.0, 463.0),
        "OUTPUT" => (800.0, 479.0),
        _ => (position.0, position.1 + radius + 38.5),
    };
    let half_spacing = if compact { 10.0 } else { 11.5 };
    let label_position = (text_center.0, text_center.1 - half_spacing);
    let value_position = (text_center.0, text_center.1 + half_spacing);
    panel_text(
        ui.painter(),
        layout.point(label_position.0, label_position.1),
        egui::Align2::CENTER_CENTER,
        label,
        display_font(
            if is_output {
                17.0
            } else if compact {
                19.0
            } else {
                26.6
            } * layout.scale,
        ),
        INK,
    );

    let value_rect = egui::Rect::from_center_size(
        layout.point(value_position.0, value_position.1),
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
                style.override_font_id = Some(egui::FontId::proportional(
                    if is_output { 14.5 } else { 16.5 } * scale,
                ));
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
                    widget.fg_stroke.color = INK;
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
    variant: usize,
    active: bool,
) {
    let (bounds, pivot, source_radius, reference_angle) = if variant == 3 {
        (
            [1524.0, 630.0, 1654.0, 760.0],
            egui::pos2(1589.0, 695.0),
            64.0,
            -32.0_f32,
        )
    } else if variant == 2 {
        (
            [1182.0, 573.0, 1362.0, 753.0],
            egui::pos2(1272.0, 663.0),
            89.0,
            40.0,
        )
    } else if variant == 1 {
        (
            [1616.0, 132.0, 1998.0, 514.0],
            egui::pos2(1807.0, 323.0),
            189.0,
            -140.0,
        )
    } else {
        (
            [620.0, 109.0, 1074.0, 565.0],
            egui::pos2(847.0, 337.0),
            226.0,
            -140.0,
        )
    };
    if variant != 3 {
        // The range stays still while the supplied cap and ivory pointer turn.
        sprite(
            painter,
            &artwork.controls,
            [28.0, 20.0, 602.0, 558.0],
            egui::pos2(331.0, 335.0),
            center,
            radius / 226.0 * 0.93,
            0.0,
        );
    }
    let angle = (135.0 + normalized.clamp(0.0, 1.0) * 270.0).to_radians();
    sprite(
        painter,
        &artwork.controls,
        bounds,
        pivot,
        center,
        radius * if variant == 3 { 1.0 } else { 0.87 } / source_radius,
        angle - reference_angle.to_radians(),
    );
    if active {
        painter.circle_stroke(
            center,
            radius * if variant == 3 { 1.0 } else { 0.87 } + 2.0,
            egui::Stroke::new(1.0_f32, ACCENT),
        );
    }
}

#[derive(Clone, Copy)]
struct PendingBypass {
    value: bool,
    requested_at: f64,
}

fn draw_bypass(
    ui: &mut egui::Ui,
    artwork: &Artwork,
    layout: PanelLayout,
    param: &BoolParam,
    setter: &ParamSetter,
) {
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
    let source = if bypassed {
        [1060.0, 140.0, 1995.0, 626.0]
    } else {
        [40.0, 140.0, 975.0, 626.0]
    };
    // Include the plate's paint splashes without expanding the click target.
    sprite(
        painter,
        &artwork.switches,
        source,
        egui::pos2(source[0] + 467.5, 359.0),
        rect.center(),
        128.0 / 800.0 * layout.scale,
        0.0,
    );
    painter.text(
        layout.point(394.0, 456.5),
        egui::Align2::CENTER_CENTER,
        if bypassed { "BYPASSED" } else { "ACTIVE" },
        display_font(if bypassed { 13.5 } else { 17.0 } * layout.scale),
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
