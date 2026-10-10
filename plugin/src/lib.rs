use nih_plug::prelude::*;
use nih_plug_egui::{create_egui_editor, egui, EguiState};
use std::sync::Arc;

struct EmberFat {
    params: Arc<EmberFatParams>,
}

#[derive(Params)]
struct EmberFatParams {
    #[persist = "editor-state"]
    editor_state: Arc<EguiState>,

    #[id = "fat"]
    fat: FloatParam,

    #[id = "color"]
    color: FloatParam,

    #[id = "mix"]
    mix: FloatParam,

    #[id = "output"]
    output: FloatParam,

    #[id = "bypass"]
    bypass: BoolParam,
}

impl Default for EmberFatParams {
    fn default() -> Self {
        Self {
            editor_state: EguiState::from_size(900, 550),
            fat: FloatParam::new(
                "Fat",
                32.0,
                FloatRange::Linear { min: 0.0, max: 100.0 },
            )
            .with_unit("%")
            .with_step_size(0.1),
            color: FloatParam::new(
                "Color",
                28.0,
                FloatRange::Linear { min: 0.0, max: 100.0 },
            )
            .with_unit("%")
            .with_step_size(0.1),
            mix: FloatParam::new(
                "Mix",
                100.0,
                FloatRange::Linear { min: 0.0, max: 100.0 },
            )
            .with_unit("%")
            .with_step_size(0.1),
            output: FloatParam::new(
                "Output",
                -1.0,
                FloatRange::Linear { min: -18.0, max: 6.0 },
            )
            .with_unit(" dB")
            .with_step_size(0.1),
            bypass: BoolParam::new("Bypass", false),
        }
    }
}

impl Default for EmberFat {
    fn default() -> Self {
        Self { params: Arc::new(EmberFatParams::default()) }
    }
}

impl Plugin for EmberFat {
    const NAME: &'static str = "EmberFat";
    const VENDOR: &'static str = "Ember Audio Lab";
    const URL: &'static str = "https://example.invalid/emberfat";
    const EMAIL: &'static str = "dev@example.invalid";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(2),
        main_output_channels: NonZeroU32::new(2),
        aux_input_ports: &[],
        aux_output_ports: &[],
        names: PortNames::const_default(),
    }];

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        let params = self.params.clone();

        create_egui_editor(
            self.params.editor_state.clone(),
            Option::<Artwork>::None,
            |ctx, artwork| {
                let mut style = (*ctx.style()).clone();
                style.visuals = egui::Visuals::dark();
                style.visuals.window_fill = egui::Color32::from_rgb(18, 18, 22);
                style.visuals.panel_fill = egui::Color32::from_rgb(18, 18, 22);
                style.visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(38, 38, 45);
                style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(65, 34, 39);
                style.visuals.widgets.active.bg_fill = egui::Color32::from_rgb(125, 28, 39);
                style.spacing.item_spacing = egui::vec2(10.0, 8.0);
                ctx.set_style(style);
                *artwork = Some(load_artwork(ctx));
            },
            move |ctx, setter, artwork| {
                let artwork = artwork
                    .as_ref()
                    .expect("EmberFat artwork is loaded during editor setup");
                egui::CentralPanel::default()
                    .frame(
                        egui::Frame::new()
                            .fill(egui::Color32::from_rgb(18, 18, 22))
                            .inner_margin(egui::Margin::same(22)),
                    )
                    .show(ctx, |ui| {
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.label(
                                    egui::RichText::new("EMBERFAT")
                                        .size(23.0)
                                        .strong()
                                        .color(egui::Color32::from_rgb(239, 70, 78)),
                                );
                                ui.label(
                                    egui::RichText::new("HARMONIC SATURATION")
                                        .size(9.0)
                                        .strong()
                                        .color(egui::Color32::from_rgb(146, 146, 154)),
                                );
                            });
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    ui.label(
                                        egui::RichText::new("STEREO  ·  VST3")
                                            .size(9.0)
                                            .strong()
                                            .color(egui::Color32::from_rgb(135, 137, 145)),
                                    );
                                },
                            );
                        });
                        ui.add_space(10.0);
                        ui.separator();
                        ui.add_space(9.0);

                        draw_front_panel(ui, artwork, &params, setter);

                        ui.add_space(8.0);
                        ui.separator();
                        ui.add_space(6.0);
                        ui.horizontal(|ui| {
                            draw_bypass_button(ui, &params.bypass, setter);
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    ui.label(
                                        egui::RichText::new(
                                            "DRAG TO ADJUST  ·  SHIFT FOR FINE CONTROL  ·  DOUBLE CLICK TO RESET",
                                        )
                                        .size(8.0)
                                        .strong()
                                        .color(egui::Color32::from_rgb(126, 127, 135)),
                                    );
                                },
                            );
                        });
                    });
            },
        )
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        _buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        true
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let fat = self.params.fat.value() / 100.0;
        let color = self.params.color.value() / 100.0;
        let mix = self.params.mix.value() / 100.0;
        let out_gain = util::db_to_gain(self.params.output.value());
        let bypass = self.params.bypass.value();

        if bypass {
            return ProcessStatus::Normal;
        }

        // Drive increases density; the normalization keeps extreme settings from
        // becoming an uncontrolled volume boost. Color adds a subtle even-harmonic
        // component before the smooth tanh curve.
        let drive = 1.0 + fat * 7.0;
        let norm = drive.tanh().max(0.0001);
        let even_harmonic = color * 0.10;

        for channel_samples in buffer.iter_samples() {
            for sample in channel_samples {
                let dry = *sample;
                let shaped = dry + even_harmonic * dry * dry * dry.signum();
                let saturated = (shaped * drive).tanh() / norm;
                let colored = saturated * (1.0 - color * 0.12)
                    + (shaped / (1.0 + shaped.abs())) * (color * 0.12);
                let wet = colored * out_gain;
                *sample = (dry + (wet - dry) * mix).clamp(-1.5, 1.5);
            }
        }

        ProcessStatus::Normal
    }
}

struct Artwork {
    knob: egui::TextureHandle,
    mascot_stages: [egui::TextureHandle; 6],
}

fn load_artwork(ctx: &egui::Context) -> Artwork {
    let knob = load_rgba_texture(
        ctx,
        "emberfat-copper-knob",
        [192, 192],
        include_bytes!("../assets/copper-knob-192.rgba"),
    );
    let mascot_stages = [
        load_rgba_texture(ctx, "fat-stage-1", [220, 352], include_bytes!("../assets/fat-stage-1.rgba")),
        load_rgba_texture(ctx, "fat-stage-2", [220, 352], include_bytes!("../assets/fat-stage-2.rgba")),
        load_rgba_texture(ctx, "fat-stage-3", [220, 352], include_bytes!("../assets/fat-stage-3.rgba")),
        load_rgba_texture(ctx, "fat-stage-4", [220, 352], include_bytes!("../assets/fat-stage-4.rgba")),
        load_rgba_texture(ctx, "fat-stage-5", [220, 352], include_bytes!("../assets/fat-stage-5.rgba")),
        load_rgba_texture(ctx, "fat-stage-6", [220, 352], include_bytes!("../assets/fat-stage-6.rgba")),
    ];
    Artwork { knob, mascot_stages }
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
    let content_height = (ui.available_height() - 48.0).clamp(360.0, 390.0);
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), content_height),
        egui::Sense::hover(),
    );
    let painter = ui.painter();
    let panel = egui::Rect::from_min_size(
        rect.left_top() + egui::vec2(3.0, 4.0),
        egui::vec2(278.0, content_height - 8.0),
    );
    painter.rect_filled(panel, 16.0, egui::Color32::from_rgb(25, 25, 30));
    painter.rect_stroke(
        panel,
        16.0,
        egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(50, 50, 57)),
        egui::StrokeKind::Inside,
    );

    let fat = (params.fat.value() / 100.0).clamp(0.0, 1.0);
    let stage = (fat * 5.0).round() as usize;
    let sprite_rect = egui::Rect::from_min_size(
        egui::pos2(panel.center().x - 105.0, panel.top() + 4.0),
        egui::vec2(210.0, 336.0),
    );
    painter.image(
        artwork.mascot_stages[stage].id(),
        sprite_rect,
        egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)),
        egui::Color32::WHITE,
    );

    const STAGE_NAMES: [&str; 6] = ["LEAN", "WARM", "THICK", "HEAVY", "FERAL", "MAXED"];
    painter.text(
        egui::pos2(panel.center().x, panel.bottom() - 20.0),
        egui::Align2::CENTER_CENTER,
        format!("FAT RESPONSE  ·  {}", STAGE_NAMES[stage]),
        egui::FontId::proportional(9.0),
        egui::Color32::from_rgb(224, 93, 87),
    );
    for index in 0..6 {
        let active = index == stage;
        painter.circle_filled(
            egui::pos2(panel.center().x - 15.0 + index as f32 * 6.0, panel.bottom() - 7.0),
            if active { 1.8 } else { 1.2 },
            if active {
                egui::Color32::from_rgb(238, 76, 79)
            } else {
                egui::Color32::from_rgb(78, 78, 84)
            },
        );
    }

    let divider_x = rect.left() + 300.0;
    painter.line_segment(
        [
            egui::pos2(divider_x, rect.top() + 18.0),
            egui::pos2(divider_x, rect.bottom() - 18.0),
        ],
        egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(48, 48, 55)),
    );

    let controls_left = rect.left() + 324.0;
    let hero_y = rect.top() + 139.0;
    draw_dial_control(
        ui,
        artwork,
        egui::pos2(controls_left + 116.0, hero_y),
        72.0,
        "FAT",
        "DRIVE",
        &params.fat,
        setter,
        false,
        egui::Color32::from_rgb(239, 67, 75),
    );
    draw_dial_control(
        ui,
        artwork,
        egui::pos2(controls_left + 365.0, hero_y),
        63.0,
        "COLOR",
        "HARMONICS",
        &params.color,
        setter,
        false,
        egui::Color32::from_rgb(205, 66, 76),
    );

    let compact_y = rect.top() + content_height - 84.0;
    draw_dial_control(
        ui,
        artwork,
        egui::pos2(controls_left + 247.0, compact_y),
        29.0,
        "MIX",
        "",
        &params.mix,
        setter,
        false,
        egui::Color32::from_rgb(224, 70, 78),
    );
    draw_dial_control(
        ui,
        artwork,
        egui::pos2(controls_left + 419.0, compact_y),
        29.0,
        "OUTPUT",
        "",
        &params.output,
        setter,
        true,
        egui::Color32::from_rgb(190, 61, 71),
    );
}

fn draw_dial_control(
    ui: &mut egui::Ui,
    artwork: &Artwork,
    center: egui::Pos2,
    radius: f32,
    label: &'static str,
    descriptor: &str,
    param: &FloatParam,
    setter: &ParamSetter,
    is_decibels: bool,
    accent: egui::Color32,
) {
    let response_rect = egui::Rect::from_center_size(
        center,
        egui::vec2(radius * 2.0 + 24.0, radius * 2.0 + 18.0),
    );
    let response = ui.interact(
        response_rect,
        egui::Id::new(("emberfat-dial", label)),
        egui::Sense::click_and_drag(),
    );
    let value = interact_float_param(ui, &response, param, setter);
    let (min_value, max_value) = parameter_bounds(param);
    let normalized = ((value - min_value) / (max_value - min_value)).clamp(0.0, 1.0);
    draw_rotary_knob(ui.painter(), &artwork.knob, center, radius, normalized, accent);

    let compact = radius < 40.0;
    let value_text = if is_decibels {
        format!("{value:+.1} dB")
    } else {
        format!("{value:.0}%")
    };
    let value_y = if compact { radius + 14.0 } else { radius + 27.0 };
    let label_y = if compact { radius + 31.0 } else { radius + 50.0 };
    painter_text(
        ui.painter(),
        center + egui::vec2(0.0, value_y),
        &value_text,
        if compact { 11.0 } else { 22.0 },
        egui::Color32::from_rgb(239, 236, 231),
        egui::FontId::proportional,
    );
    painter_text(
        ui.painter(),
        center + egui::vec2(0.0, label_y),
        label,
        if compact { 9.0 } else { 12.0 },
        egui::Color32::from_rgb(231, 228, 222),
        egui::FontId::proportional,
    );
    if !descriptor.is_empty() {
        painter_text(
            ui.painter(),
            center + egui::vec2(0.0, radius + 68.0),
            descriptor,
            8.0,
            egui::Color32::from_rgb(132, 132, 140),
            egui::FontId::proportional,
        );
    }
}

fn painter_text(
    painter: &egui::Painter,
    position: egui::Pos2,
    text: &str,
    size: f32,
    color: egui::Color32,
    font: fn(f32) -> egui::FontId,
) {
    painter.text(
        position,
        egui::Align2::CENTER_CENTER,
        text,
        font(size),
        color,
    );
}

fn draw_rotary_knob(
    painter: &egui::Painter,
    texture: &egui::TextureHandle,
    center: egui::Pos2,
    radius: f32,
    normalized: f32,
    accent: egui::Color32,
) {
    let shadow = center + egui::vec2(0.0, radius * 0.10);
    painter.circle_filled(shadow, radius + 9.0, egui::Color32::from_rgb(8, 8, 10));
    painter.circle_filled(center, radius + 7.0, egui::Color32::from_rgb(24, 24, 29));
    painter.circle_stroke(
        center,
        radius + 6.5,
        egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(86, 86, 93)),
    );

    let start_angle = std::f32::consts::PI * 0.75;
    let sweep = std::f32::consts::PI * 1.5;
    let mut track = Vec::with_capacity(41);
    let mut active = Vec::with_capacity(41);
    for step in 0..=40 {
        let t = step as f32 / 40.0;
        let angle = start_angle + sweep * t;
        let point = center + egui::vec2((radius + 12.0) * angle.cos(), (radius + 12.0) * angle.sin());
        track.push(point);
        if t <= normalized {
            active.push(point);
        }
    }
    painter.add(egui::Shape::line(
        track,
        egui::Stroke::new(2.0_f32, egui::Color32::from_rgb(72, 43, 48)),
    ));
    if active.len() > 1 {
        painter.add(egui::Shape::line(active, egui::Stroke::new(3.0_f32, accent)));
    }

    let tick_count = if radius < 40.0 { 24 } else { 36 };
    for tick in 0..=tick_count {
        let t = tick as f32 / tick_count as f32;
        let angle = start_angle + sweep * t;
        let inner_radius = radius + 7.0;
        let outer_radius = radius + if tick % 3 == 0 { 15.0 } else { 12.0 };
        let inner = center + egui::vec2(inner_radius * angle.cos(), inner_radius * angle.sin());
        let outer = center + egui::vec2(outer_radius * angle.cos(), outer_radius * angle.sin());
        let color = if t <= normalized {
            accent
        } else if tick % 3 == 0 {
            egui::Color32::from_rgb(130, 130, 137)
        } else {
            egui::Color32::from_rgb(76, 76, 83)
        };
        painter.line_segment(
            [inner, outer],
            egui::Stroke::new(if tick % 3 == 0 { 1.4_f32 } else { 0.9_f32 }, color),
        );
    }

    let target_pointer_angle = start_angle + normalized * sweep;
    let source_pointer_angle = -1.0_f32;
    draw_rotated_texture(
        painter,
        texture,
        center,
        radius * 2.0,
        target_pointer_angle - source_pointer_angle,
    );
}

fn draw_rotated_texture(
    painter: &egui::Painter,
    texture: &egui::TextureHandle,
    center: egui::Pos2,
    size: f32,
    angle: f32,
) {
    let half = size * 0.5;
    let sin = angle.sin();
    let cos = angle.cos();
    let corners = [
        (egui::vec2(-half, -half), egui::pos2(0.0, 0.0)),
        (egui::vec2(half, -half), egui::pos2(1.0, 0.0)),
        (egui::vec2(half, half), egui::pos2(1.0, 1.0)),
        (egui::vec2(-half, half), egui::pos2(0.0, 1.0)),
    ];
    let mut mesh = egui::Mesh::with_texture(texture.id());
    for (offset, uv) in corners {
        let rotated = egui::vec2(
            offset.x * cos - offset.y * sin,
            offset.x * sin + offset.y * cos,
        );
        mesh.vertices.push(egui::epaint::Vertex {
            pos: center + rotated,
            uv,
            color: egui::Color32::WHITE,
        });
    }
    mesh.indices.extend([0, 1, 2, 0, 2, 3]);
    painter.add(egui::Shape::mesh(mesh));
}

fn draw_bypass_button(ui: &mut egui::Ui, bypass: &BoolParam, setter: &ParamSetter) {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(98.0, 26.0), egui::Sense::click());
    if response.clicked() {
        setter.begin_set_parameter(bypass);
        setter.set_parameter(bypass, !bypass.value());
        setter.end_set_parameter(bypass);
    }
    let active = bypass.value();
    let fill = if active {
        egui::Color32::from_rgb(103, 29, 37)
    } else {
        egui::Color32::from_rgb(32, 32, 38)
    };
    let outline = if active {
        egui::Color32::from_rgb(179, 54, 61)
    } else {
        egui::Color32::from_rgb(61, 61, 68)
    };
    let painter = ui.painter();
    painter.rect_filled(rect, 13.0, fill);
    painter.rect_stroke(rect, 13.0, egui::Stroke::new(1.0_f32, outline), egui::StrokeKind::Inside);
    painter.circle_filled(
        egui::pos2(rect.left() + 14.0, rect.center().y),
        3.2,
        if active {
            egui::Color32::from_rgb(255, 111, 105)
        } else {
            egui::Color32::from_rgb(94, 94, 101)
        },
    );
    painter.text(
        egui::pos2(rect.left() + 26.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        "BYPASS",
        egui::FontId::proportional(9.0),
        egui::Color32::from_rgb(225, 222, 216),
    );
}

fn parameter_bounds(param: &FloatParam) -> (f32, f32) {
    match param.range() {
        FloatRange::Linear { min, max } => (min, max),
        FloatRange::Skewed { min, max, .. } => (min, max),
        FloatRange::SymmetricalSkewed { min, max, .. } => (min, max),
        FloatRange::Reversed(inner) => match *inner {
            FloatRange::Linear { min, max } => (min, max),
            FloatRange::Skewed { min, max, .. } => (min, max),
            FloatRange::SymmetricalSkewed { min, max, .. } => (min, max),
            FloatRange::Reversed(_) => unreachable!("Nested reversed FloatRange"),
        },
    }
}

fn interact_float_param(
    ui: &egui::Ui,
    response: &egui::Response,
    param: &FloatParam,
    setter: &ParamSetter,
) -> f32 {
    let (min_value, max_value) = parameter_bounds(param);
    let mut value = param.value();
    if response.drag_started() {
        setter.begin_set_parameter(param);
    }
    if response.dragged() {
        let sensitivity = if ui.input(|input| input.modifiers.shift) { 0.1 } else { 1.0 };
        let delta = -response.drag_motion().y * (max_value - min_value) / 480.0 * sensitivity;
        value = (value + delta).clamp(min_value, max_value);
        setter.set_parameter(param, value);
    }
    if response.drag_stopped() {
        setter.end_set_parameter(param);
    }
    if response.double_clicked() {
        setter.begin_set_parameter(param);
        value = param.default_plain_value();
        setter.set_parameter(param, value);
        setter.end_set_parameter(param);
    }
    value
}

impl Vst3Plugin for EmberFat {
    const VST3_CLASS_ID: [u8; 16] = *b"EmberFatPlugin01";

    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Distortion];
}

nih_export_vst3!(EmberFat);
