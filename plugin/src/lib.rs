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
            editor_state: EguiState::from_size(800, 500),
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
            (),
            |ctx, _| {
                let mut style = (*ctx.style()).clone();
                style.visuals = egui::Visuals::dark();
                style.visuals.window_fill = egui::Color32::from_rgb(18, 18, 22);
                style.visuals.panel_fill = egui::Color32::from_rgb(18, 18, 22);
                style.visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(38, 38, 45);
                style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(65, 34, 39);
                style.visuals.widgets.active.bg_fill = egui::Color32::from_rgb(125, 28, 39);
                style.spacing.item_spacing = egui::vec2(10.0, 8.0);
                ctx.set_style(style);
            },
            move |ctx, setter, _| {
                egui::CentralPanel::default()
                    .frame(egui::Frame::new().fill(egui::Color32::from_rgb(18, 18, 22)).inner_margin(egui::Margin::same(20)))
                    .show(ctx, |ui| {
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.label(egui::RichText::new("EMBERFAT").size(23.0).strong().color(egui::Color32::from_rgb(239, 70, 78)));
                                ui.label(egui::RichText::new("HARMONIC SATURATION").size(9.0).strong().color(egui::Color32::from_rgb(146, 146, 154)));
                            });
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(egui::RichText::new("STEREO  •  VST3").size(9.0).strong().color(egui::Color32::from_rgb(135, 137, 145)));
                            });
                        });
                        ui.add_space(12.0);
                        let line_y = ui.cursor().top();
                        ui.painter().line_segment(
                            [egui::pos2(ui.min_rect().left(), line_y), egui::pos2(ui.min_rect().right(), line_y)],
                            egui::Stroke::new(1.0, egui::Color32::from_rgb(59, 59, 65)),
                        );
                        ui.add_space(10.0);

                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 14.0;
                            draw_ember_mascot(ui, params.fat.value() / 100.0, params.color.value() / 100.0);
                            ui.vertical(|ui| {
                                ui.horizontal(|ui| {
                                    draw_hero_knob(ui, "FAT", "DRIVE", &params.fat, setter, egui::Color32::from_rgb(246, 64, 73));
                                    draw_hero_knob(ui, "COLOR", "HARMONICS", &params.color, setter, egui::Color32::from_rgb(205, 72, 82));
                                });
                                ui.add_space(9.0);
                                ui.horizontal(|ui| {
                                    draw_small_control(ui, "MIX", "DRY / WET", &params.mix, setter, egui::Color32::from_rgb(228, 67, 76));
                                    draw_small_control(ui, "OUTPUT", "LEVEL", &params.output, setter, egui::Color32::from_rgb(187, 59, 68));
                                });
                            });
                        });

                        ui.add_space(9.0);
                        let line_y = ui.cursor().top();
                        ui.painter().line_segment(
                            [egui::pos2(ui.min_rect().left(), line_y), egui::pos2(ui.min_rect().right(), line_y)],
                            egui::Stroke::new(1.0, egui::Color32::from_rgb(59, 59, 65)),
                        );
                        ui.add_space(7.0);
                        ui.horizontal(|ui| {
                            let mut bypass = params.bypass.value();
                            let response = ui.checkbox(&mut bypass, "BYPASS EFFECT");
                            if response.changed() {
                                setter.begin_set_parameter(&params.bypass);
                                setter.set_parameter(&params.bypass, bypass);
                                setter.end_set_parameter(&params.bypass);
                            }
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(egui::RichText::new("DRAG TO ADJUST  ·  SHIFT FOR FINE CONTROL  ·  DOUBLE CLICK TO RESET").size(8.0).strong().color(egui::Color32::from_rgb(126, 127, 135)));
                            });
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

fn draw_hero_knob(
    ui: &mut egui::Ui,
    label: &str,
    descriptor: &str,
    param: &FloatParam,
    setter: &ParamSetter,
    accent: egui::Color32,
) {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(264.0, 244.0), egui::Sense::click_and_drag());
    let painter = ui.painter();
    let card = if response.hovered() { egui::Color32::from_rgb(35, 34, 39) } else { egui::Color32::from_rgb(27, 27, 32) };
    painter.rect_filled(rect, 12.0, card);
    painter.rect_stroke(
        rect,
        12.0,
        egui::Stroke::new(1.0, if response.hovered() { egui::Color32::from_rgb(105, 51, 56) } else { egui::Color32::from_rgb(52, 52, 59) }),
        egui::StrokeKind::Inside,
    );

    let value = interact_float_param(ui, &response, param, setter);
    let (min_value, max_value) = parameter_bounds(param);
    let normalized = ((value - min_value) / (max_value - min_value)).clamp(0.0, 1.0);
    let center = egui::pos2(rect.center().x, rect.top() + 91.0);
    draw_realistic_dial(painter, center, 55.0, normalized, accent);

    let value_text = if label == "OUTPUT" { format!("{value:.1} dB") } else { format!("{value:.0}%") };
    painter.text(
        egui::pos2(rect.center().x, rect.top() + 177.0),
        egui::Align2::CENTER_CENTER,
        value_text,
        egui::FontId::proportional(21.0),
        egui::Color32::from_rgb(238, 235, 229),
    );
    painter.text(
        egui::pos2(rect.center().x, rect.top() + 207.0),
        egui::Align2::CENTER_CENTER,
        label,
        egui::FontId::proportional(13.0),
        egui::Color32::from_rgb(242, 239, 232),
    );
    painter.text(
        egui::pos2(rect.center().x, rect.top() + 228.0),
        egui::Align2::CENTER_CENTER,
        descriptor,
        egui::FontId::proportional(9.0),
        egui::Color32::from_rgb(139, 139, 147),
    );
}

fn draw_small_control(
    ui: &mut egui::Ui,
    label: &str,
    descriptor: &str,
    param: &FloatParam,
    setter: &ParamSetter,
    accent: egui::Color32,
) {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(264.0, 82.0), egui::Sense::click_and_drag());
    let painter = ui.painter();
    let card = if response.hovered() { egui::Color32::from_rgb(35, 34, 39) } else { egui::Color32::from_rgb(27, 27, 32) };
    painter.rect_filled(rect, 10.0, card);
    painter.rect_stroke(
        rect,
        10.0,
        egui::Stroke::new(1.0, if response.hovered() { egui::Color32::from_rgb(105, 51, 56) } else { egui::Color32::from_rgb(52, 52, 59) }),
        egui::StrokeKind::Inside,
    );

    let value = interact_float_param(ui, &response, param, setter);
    let (min_value, max_value) = parameter_bounds(param);
    let normalized = ((value - min_value) / (max_value - min_value)).clamp(0.0, 1.0);
    let center = egui::pos2(rect.left() + 47.0, rect.center().y);
    draw_realistic_dial(painter, center, 23.0, normalized, accent);

    let value_text = if label == "OUTPUT" { format!("{value:.1} dB") } else { format!("{value:.0}%") };
    painter.text(
        egui::pos2(rect.left() + 88.0, rect.center().y - 8.0),
        egui::Align2::LEFT_CENTER,
        value_text,
        egui::FontId::proportional(16.0),
        egui::Color32::from_rgb(238, 235, 229),
    );
    painter.text(
        egui::pos2(rect.right() - 14.0, rect.center().y - 11.0),
        egui::Align2::RIGHT_CENTER,
        label,
        egui::FontId::proportional(10.0),
        egui::Color32::from_rgb(215, 213, 208),
    );
    painter.text(
        egui::pos2(rect.right() - 14.0, rect.center().y + 10.0),
        egui::Align2::RIGHT_CENTER,
        descriptor,
        egui::FontId::proportional(8.0),
        egui::Color32::from_rgb(129, 130, 138),
    );
}

fn draw_realistic_dial(
    painter: &egui::Painter,
    center: egui::Pos2,
    radius: f32,
    normalized: f32,
    accent: egui::Color32,
) {
    let shadow = center + egui::vec2(0.0, radius * 0.12);
    painter.circle_filled(shadow, radius + 13.0, egui::Color32::from_rgb(10, 10, 12));
    painter.circle_filled(center, radius + 12.0, egui::Color32::from_rgb(24, 25, 29));
    painter.circle_stroke(center, radius + 11.0, egui::Stroke::new(1.0, egui::Color32::from_rgb(119, 119, 126)));
    painter.circle_filled(center, radius + 7.0, egui::Color32::from_rgb(71, 72, 78));
    painter.circle_filled(center, radius + 3.0, egui::Color32::from_rgb(31, 32, 36));

    // Fine radial grooves give the rim a machined, knurled edge.
    for tick in 0..48 {
        let angle = std::f32::consts::TAU * tick as f32 / 48.0;
        let outer = radius + 9.0;
        let inner = radius + if tick % 4 == 0 { 3.0 } else { 5.5 };
        let color = if tick % 4 == 0 { egui::Color32::from_rgb(154, 154, 159) } else { egui::Color32::from_rgb(88, 89, 95) };
        painter.line_segment(
            [
                center + egui::vec2(angle.cos() * inner, angle.sin() * inner),
                center + egui::vec2(angle.cos() * outer, angle.sin() * outer),
            ],
            egui::Stroke::new(if tick % 4 == 0 { 1.4 } else { 0.8 }, color),
        );
    }

    painter.circle_filled(center, radius, egui::Color32::from_rgb(43, 44, 49));
    painter.circle_stroke(center, radius - 1.0, egui::Stroke::new(1.0, egui::Color32::from_rgb(13, 14, 16)));
    painter.circle_filled(center, radius - 5.0, egui::Color32::from_rgb(29, 30, 34));
    painter.circle_stroke(center, radius - 8.0, egui::Stroke::new(1.0, egui::Color32::from_rgb(65, 66, 72)));

    // Soft specular arcs make the black face read like polished metal.
    for (start, end, color, width) in [
        (3.55_f32, 4.75_f32, egui::Color32::from_rgb(118, 119, 125), 2.0),
        (4.95_f32, 5.45_f32, egui::Color32::from_rgb(78, 79, 85), 1.0),
    ] {
        let mut points = Vec::new();
        for step in 0..=18 {
            let angle = start + (end - start) * step as f32 / 18.0;
            points.push(center + egui::vec2((radius - 11.0) * angle.cos(), (radius - 11.0) * angle.sin()));
        }
        painter.add(egui::Shape::line(points, egui::Stroke::new(width, color)));
    }

    let start_angle = std::f32::consts::PI * 0.75;
    let sweep = std::f32::consts::PI * 1.5;
    let mut track = Vec::new();
    let mut active = Vec::new();
    let steps = 48;
    let active_steps = (normalized * steps as f32).round() as usize;
    for step in 0..=steps {
        let t = step as f32 / steps as f32;
        let angle = start_angle + t * sweep;
        let point = center + egui::vec2((radius + 15.0) * angle.cos(), (radius + 15.0) * angle.sin());
        track.push(point);
        if step <= active_steps {
            active.push(point);
        }
    }
    painter.add(egui::Shape::line(track, egui::Stroke::new(2.0, egui::Color32::from_rgb(74, 43, 48))));
    if active.len() > 1 {
        painter.add(egui::Shape::line(active, egui::Stroke::new(3.2, accent)));
    }

    let angle = start_angle + normalized * sweep;
    let pointer_end = center + egui::vec2((radius - 13.0) * angle.cos(), (radius - 13.0) * angle.sin());
    painter.line_segment([center, pointer_end], egui::Stroke::new(3.4, accent));
    painter.circle_filled(center, radius * 0.16, egui::Color32::from_rgb(129, 130, 136));
    painter.circle_filled(center, radius * 0.09, egui::Color32::from_rgb(226, 222, 216));
}

fn draw_ember_mascot(ui: &mut egui::Ui, fat: f32, color: f32) {
    let fat = fat.clamp(0.0, 1.0);
    let color = color.clamp(0.0, 1.0);
    let (rect, _) = ui.allocate_exact_size(egui::vec2(190.0, 335.0), egui::Sense::hover());
    let painter = ui.painter();
    painter.rect_filled(rect, 12.0, egui::Color32::from_rgb(24, 24, 29));
    painter.rect_stroke(rect, 12.0, egui::Stroke::new(1.0, egui::Color32::from_rgb(54, 54, 61)), egui::StrokeKind::Inside);

    let center = egui::pos2(rect.center().x, rect.top() + 153.0);
    let rx = 31.0 + fat * 17.0;
    let ry = 61.0 + fat * 15.0;
    painter.circle_filled(center, 61.0 + fat * 20.0, egui::Color32::from_rgba_unmultiplied(166, 33, 42, 22));

    // A pair of tiny ember wheels rotate their spokes with COLOR.
    for side in [-1.0_f32, 1.0_f32] {
        let wheel = center + egui::vec2(side * (rx + 17.0), 12.0);
        painter.circle_filled(wheel, 14.0, egui::Color32::from_rgb(20, 21, 24));
        painter.circle_stroke(wheel, 13.0, egui::Stroke::new(2.0, egui::Color32::from_rgb(123, 124, 129)));
        painter.circle_stroke(wheel, 8.0, egui::Stroke::new(1.0, egui::Color32::from_rgb(79, 80, 86)));
        for spoke in 0..6 {
            let angle = color * std::f32::consts::TAU + spoke as f32 * std::f32::consts::TAU / 6.0;
            let start = wheel + egui::vec2(angle.cos() * 3.0, angle.sin() * 3.0);
            let end = wheel + egui::vec2(angle.cos() * 10.0, angle.sin() * 10.0);
            painter.line_segment([start, end], egui::Stroke::new(1.0, egui::Color32::from_rgb(188, 67, 72)));
        }
        painter.circle_filled(wheel, 2.5, egui::Color32::from_rgb(205, 205, 207));
    }

    let flame = vec![
        center + egui::vec2(-rx * 0.58, -ry * 0.48),
        center + egui::vec2(-rx * 0.55, -ry - 22.0),
        center + egui::vec2(-rx * 0.08, -ry * 0.68),
        center + egui::vec2(rx * 0.40, -ry - 15.0),
        center + egui::vec2(rx * 0.56, -ry * 0.46),
    ];
    painter.add(egui::Shape::convex_polygon(
        flame,
        egui::Color32::from_rgb(201, 47, 53),
        egui::Stroke::new(1.5, egui::Color32::from_rgb(239, 81, 79)),
    ));

    let mut body = Vec::new();
    for step in 0..48 {
        let angle = std::f32::consts::TAU * step as f32 / 48.0;
        let wobble = 1.0 + 0.06 * (3.0 * angle + fat * 1.4).sin() + 0.025 * (5.0 * angle).cos();
        body.push(center + egui::vec2(rx * angle.cos() * wobble, ry * angle.sin() * wobble));
    }
    let body_color = egui::Color32::from_rgb(170 + (fat * 34.0) as u8, 31 + (fat * 27.0) as u8, 38 + (fat * 8.0) as u8);
    painter.add(egui::Shape::convex_polygon(
        body,
        body_color,
        egui::Stroke::new(2.0, egui::Color32::from_rgb(244, 91, 77)),
    ));

    // Warm belly highlight and four expressions that progress with FAT.
    painter.circle_filled(center + egui::vec2(0.0, 30.0), 13.0 + fat * 8.0, egui::Color32::from_rgba_unmultiplied(255, 132, 73, 105));
    let expression = (fat * 4.0).round() as u8;
    let eye_y = center.y - 10.0 - fat * 4.0;
    let eye_spacing = 11.0 + fat * 2.0;
    let eye_radius = 5.0 + fat * 1.6;
    for side in [-1.0_f32, 1.0_f32] {
        let eye = egui::pos2(center.x + side * eye_spacing, eye_y);
        painter.circle_filled(eye, eye_radius + 2.0, egui::Color32::from_rgb(249, 238, 219));
        painter.circle_filled(eye + egui::vec2(1.0, 1.0), eye_radius * 0.48, egui::Color32::from_rgb(28, 23, 24));
        painter.circle_filled(eye + egui::vec2(2.3, -1.3), 1.5, egui::Color32::WHITE);
        painter.circle_filled(eye + egui::vec2(side * 10.0, 11.0), 3.0 + fat, egui::Color32::from_rgba_unmultiplied(255, 131, 114, 110));
    }
    if expression >= 3 {
        painter.circle_filled(center + egui::vec2(0.0, 15.0), 5.5 + fat * 2.0, egui::Color32::from_rgb(58, 17, 27));
        painter.circle_stroke(center + egui::vec2(0.0, 15.0), 5.5 + fat * 2.0, egui::Stroke::new(1.0, egui::Color32::from_rgb(255, 137, 116)));
    } else {
        let smile = [
            center + egui::vec2(-8.0, 15.0),
            center + egui::vec2(-4.0, 20.0 + fat * 2.0),
            center + egui::vec2(2.0, 21.0 + fat * 2.0),
            center + egui::vec2(8.0, 15.0),
        ];
        painter.add(egui::Shape::line(smile.to_vec(), egui::Stroke::new(2.0, egui::Color32::from_rgb(54, 17, 24))));
    }

    let mood = match expression {
        0 => "GLOWING",
        1 => "WARMING UP",
        2 => "GETTING FAT",
        3 => "BLAZING",
        _ => "MAXED OUT",
    };
    painter.text(
        egui::pos2(rect.center().x, rect.bottom() - 35.0),
        egui::Align2::CENTER_CENTER,
        "EMBER RESPONSE",
        egui::FontId::proportional(9.0),
        egui::Color32::from_rgb(141, 141, 149),
    );
    painter.text(
        egui::pos2(rect.center().x, rect.bottom() - 17.0),
        egui::Align2::CENTER_CENTER,
        mood,
        egui::FontId::proportional(10.0),
        egui::Color32::from_rgb(236, 92, 83),
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
        let delta = -response.drag_motion().y * (max_value - min_value) / 360.0 * sensitivity;
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

