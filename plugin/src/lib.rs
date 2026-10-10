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
            editor_state: EguiState::from_size(1000, 540),
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
                style.visuals.window_fill = egui::Color32::from_rgb(24, 19, 16);
                style.visuals.panel_fill = egui::Color32::from_rgb(24, 19, 16);
                ctx.set_style(style);
                *artwork = Some(load_artwork(ctx));
            },
            move |ctx, setter, artwork| {
                let artwork = artwork
                    .as_ref()
                    .expect("EmberFat artwork is loaded during editor setup");
                egui::CentralPanel::default()
                    .frame(egui::Frame::new().fill(egui::Color32::from_rgb(24, 19, 16)))
                    .show(ctx, |ui| {
                        draw_front_panel(ui, artwork, &params, setter);
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
    background: egui::TextureHandle,
    knob: egui::TextureHandle,
    knob_range: egui::TextureHandle,
    output_knob: egui::TextureHandle,
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
        "emberfat-red-metal-knob",
        [256, 256],
        include_bytes!("../assets/red-metal-knob-256.rgba"),
    );
    let knob_range = load_rgba_texture(
        ctx,
        "emberfat-knob-range",
        [256, 256],
        include_bytes!("../assets/knob-range-256.rgba"),
    );
    let output_knob = load_rgba_texture(
        ctx,
        "emberfat-output-knob",
        [72, 72],
        include_bytes!("../assets/output-knob-72.rgba"),
    );
    let mascot_stages = [
        load_rgba_texture(ctx, "fat-stage-1", [240, 390], include_bytes!("../assets/fat-stage-1.rgba")),
        load_rgba_texture(ctx, "fat-stage-2", [240, 390], include_bytes!("../assets/fat-stage-2.rgba")),
        load_rgba_texture(ctx, "fat-stage-3", [240, 390], include_bytes!("../assets/fat-stage-3.rgba")),
        load_rgba_texture(ctx, "fat-stage-4", [240, 390], include_bytes!("../assets/fat-stage-4.rgba")),
        load_rgba_texture(ctx, "fat-stage-5", [240, 390], include_bytes!("../assets/fat-stage-5.rgba")),
        load_rgba_texture(ctx, "fat-stage-6", [240, 390], include_bytes!("../assets/fat-stage-6.rgba")),
    ];
    Artwork {
        background,
        knob,
        knob_range,
        output_knob,
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

fn draw_front_panel(
    ui: &mut egui::Ui,
    artwork: &Artwork,
    params: &EmberFatParams,
    setter: &ParamSetter,
) {
    let rect = ui.max_rect();
    let painter = ui.painter();
    painter.rect_filled(rect, 0.0, egui::Color32::from_rgb(24, 19, 16));
    painter.image(
        artwork.background.id(),
        rect,
        egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)),
        egui::Color32::WHITE,
    );

    draw_heading(
        painter,
        egui::pos2(rect.left() + 112.0, rect.top() + 98.0),
        "EMBERFAT",
        22.0,
        egui::Color32::from_rgb(246, 228, 202),
    );
    draw_heading(
        painter,
        egui::pos2(rect.left() + 113.0, rect.top() + 120.0),
        "HARMONIC SATURATION",
        9.0,
        egui::Color32::from_rgb(224, 204, 177),
    );

    let fat = (params.fat.value() / 100.0).clamp(0.0, 1.0);
    let stage = (fat * 5.0).round() as usize;
    const STAGE_NAMES: [&str; 6] = ["LEAN", "WARM", "THICK", "HEAVY", "FERAL", "MAXED"];
    draw_heading(
        painter,
        egui::pos2(rect.left() + 113.0, rect.top() + 142.0),
        &format!("FAT RESPONSE  ·  {}", STAGE_NAMES[stage]),
        8.0,
        egui::Color32::from_rgb(236, 152, 115),
    );
    let sprite_rect = egui::Rect::from_min_size(
        egui::pos2(rect.left() + 70.0, rect.top() + 149.0),
        egui::vec2(196.0, 319.0),
    );
    painter.image(
        artwork.mascot_stages[stage].id(),
        sprite_rect,
        egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)),
        egui::Color32::WHITE,
    );

    draw_dial_control(
        ui,
        artwork,
        egui::pos2(rect.left() + 440.0, rect.top() + 220.0),
        53.0,
        "FAT",
        &params.fat,
        setter,
        false,
        false,
    );
    draw_dial_control(
        ui,
        artwork,
        egui::pos2(rect.left() + 660.0, rect.top() + 238.0),
        51.0,
        "COLOR",
        &params.color,
        setter,
        false,
        false,
    );
    draw_dial_control(
        ui,
        artwork,
        egui::pos2(rect.left() + 520.0, rect.top() + 386.0),
        26.0,
        "MIX",
        &params.mix,
        setter,
        false,
        false,
    );
    draw_dial_control(
        ui,
        artwork,
        egui::pos2(rect.left() + 766.0, rect.top() + 386.0),
        18.0,
        "OUTPUT",
        &params.output,
        setter,
        true,
        true,
    );

    draw_bypass_button(
        ui,
        egui::pos2(rect.left() + 333.0, rect.top() + 410.0),
        &params.bypass,
        setter,
    );
}

fn draw_heading(
    painter: &egui::Painter,
    position: egui::Pos2,
    text: &str,
    size: f32,
    color: egui::Color32,
) {
    painter.text(
        position + egui::vec2(0.8, 1.2),
        egui::Align2::LEFT_CENTER,
        text,
        egui::FontId::proportional(size),
        egui::Color32::from_rgba_unmultiplied(18, 12, 10, 210),
    );
    painter.text(
        position,
        egui::Align2::LEFT_CENTER,
        text,
        egui::FontId::proportional(size),
        color,
    );
}

fn draw_dial_control(
    ui: &mut egui::Ui,
    artwork: &Artwork,
    center: egui::Pos2,
    radius: f32,
    label: &'static str,
    param: &FloatParam,
    setter: &ParamSetter,
    is_decibels: bool,
    is_output: bool,
) {
    let response_rect = egui::Rect::from_center_size(
        center,
        egui::vec2(radius * 2.0 + 30.0, radius * 2.0 + 22.0),
    );
    let response = ui.interact(
        response_rect,
        egui::Id::new(("emberfat-dial", label)),
        egui::Sense::click_and_drag(),
    );
    let value = interact_float_param(ui, &response, param, setter);
    let (min_value, max_value) = parameter_bounds(param);
    let normalized = ((value - min_value) / (max_value - min_value)).clamp(0.0, 1.0);
    draw_rotary_knob(ui.painter(), artwork, center, radius, normalized, is_output);

    let compact = radius < 40.0;
    let value_text = if is_decibels {
        format!("{value:+.1} dB")
    } else {
        format!("{value:.0}%")
    };
    let (value_offset, label_offset, value_size, label_size) = if is_output {
        (radius + 8.0, radius + 25.0, 9.0, 8.0)
    } else if compact {
        (radius + 20.0, radius + 37.0, 10.0, 9.0)
    } else {
        (radius + 19.0, radius + 39.0, 14.0, 11.0)
    };
    painter_text(
        ui.painter(),
        center + egui::vec2(0.0, value_offset),
        &value_text,
        value_size,
        egui::Color32::from_rgb(249, 237, 218),
        egui::FontId::proportional,
    );
    painter_text(
        ui.painter(),
        center + egui::vec2(0.0, label_offset),
        label,
        label_size,
        egui::Color32::from_rgb(244, 223, 195),
        egui::FontId::proportional,
    );
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
        position + egui::vec2(0.6, 0.8),
        egui::Align2::CENTER_CENTER,
        text,
        font(size),
        egui::Color32::from_rgba_unmultiplied(18, 12, 10, 210),
    );
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
    artwork: &Artwork,
    center: egui::Pos2,
    radius: f32,
    normalized: f32,
    is_output: bool,
) {
    let texture = if is_output {
        &artwork.output_knob
    } else {
        let range_diameter = radius * 2.0 + 26.0;
        let range_rect = egui::Rect::from_center_size(
            center,
            egui::vec2(range_diameter, range_diameter),
        );
        painter.image(
            artwork.knob_range.id(),
            range_rect,
            egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)),
            egui::Color32::WHITE,
        );
        &artwork.knob
    };

    let start_angle = std::f32::consts::PI * 0.75;
    let sweep = std::f32::consts::PI * 1.5;
    let target_pointer_angle = start_angle + normalized * sweep;
    let source_pointer_angle = if is_output {
        -1.0_f32
    } else {
        -std::f32::consts::FRAC_PI_2
    };
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

fn draw_bypass_button(
    ui: &mut egui::Ui,
    center: egui::Pos2,
    bypass: &BoolParam,
    setter: &ParamSetter,
) {
    let rect = egui::Rect::from_center_size(center, egui::vec2(88.0, 26.0));
    let response = ui.interact(
        rect,
        egui::Id::new("emberfat-bypass"),
        egui::Sense::click(),
    );
    if response.clicked() {
        setter.begin_set_parameter(bypass);
        setter.set_parameter(bypass, !bypass.value());
        setter.end_set_parameter(bypass);
    }
    let active = bypass.value();
    let fill = if active {
        egui::Color32::from_rgb(131, 47, 43)
    } else {
        egui::Color32::from_rgb(54, 40, 31)
    };
    let painter = ui.painter();
    painter.rect_filled(rect, 13.0, fill);
    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        "BYPASS",
        egui::FontId::proportional(9.0),
        egui::Color32::from_rgb(249, 229, 205),
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
        let delta = -response.drag_motion().y * (max_value - min_value) / 160.0 * sensitivity;
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
