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
            editor_state: EguiState::from_size(720, 430),
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
                style.visuals.window_fill = egui::Color32::from_rgb(19, 20, 23);
                style.visuals.panel_fill = egui::Color32::from_rgb(19, 20, 23);
                style.visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(38, 39, 43);
                style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(75, 40, 44);
                style.visuals.widgets.active.bg_fill = egui::Color32::from_rgb(142, 42, 49);
                style.spacing.item_spacing = egui::vec2(12.0, 10.0);
                ctx.set_style(style);
            },
            move |ctx, setter, _| {
                egui::CentralPanel::default()
                    .frame(egui::Frame::new().fill(egui::Color32::from_rgb(19, 20, 23)).inner_margin(egui::Margin::same(22)))
                    .show(ctx, |ui| {
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.label(egui::RichText::new("EMBERFAT").size(25.0).strong().color(egui::Color32::from_rgb(238, 82, 86)));
                                ui.label(egui::RichText::new("HARMONIC SATURATION  /  VST3 EFFECT").size(10.0).strong().color(egui::Color32::from_rgb(139, 142, 148)));
                            });
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(egui::RichText::new("●  STEREO").size(10.0).strong().color(egui::Color32::from_rgb(164, 169, 164)));
                            });
                        });
                        ui.add_space(15.0);
                        let line_y = ui.cursor().top();
                        ui.painter().line_segment(
                            [egui::pos2(ui.min_rect().left(), line_y), egui::pos2(ui.min_rect().right(), line_y)],
                            egui::Stroke::new(1.0, egui::Color32::from_rgb(54, 55, 59)),
                        );
                        ui.add_space(13.0);

                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 12.0;
                            draw_knob(ui, "FAT", "DRIVE", &params.fat, setter, egui::Color32::from_rgb(239, 77, 81));
                            draw_knob(ui, "COLOR", "HARMONICS", &params.color, setter, egui::Color32::from_rgb(226, 91, 82));
                            draw_knob(ui, "MIX", "DRY / WET", &params.mix, setter, egui::Color32::from_rgb(205, 75, 79));
                            draw_knob(ui, "OUTPUT", "LEVEL", &params.output, setter, egui::Color32::from_rgb(181, 67, 72));
                        });

                        ui.add_space(9.0);
                        let line_y = ui.cursor().top();
                        ui.painter().line_segment(
                            [egui::pos2(ui.min_rect().left(), line_y), egui::pos2(ui.min_rect().right(), line_y)],
                            egui::Stroke::new(1.0, egui::Color32::from_rgb(54, 55, 59)),
                        );
                        ui.add_space(7.0);
                        ui.horizontal(|ui| {
                            let mut bypass = params.bypass.value();
                            let response = ui.checkbox(&mut bypass, "BYPASS EFFECT");
                            if response.changed() {
                                setter.set_parameter(&params.bypass, bypass);
                            }
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(egui::RichText::new("DOUBLE CLICK A CONTROL TO RESET").size(9.0).strong().color(egui::Color32::from_rgb(111, 114, 120)));
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

fn draw_knob(
    ui: &mut egui::Ui,
    label: &str,
    descriptor: &str,
    param: &FloatParam,
    setter: &ParamSetter,
    accent: egui::Color32,
) {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(155.0, 218.0), egui::Sense::click_and_drag());
    let painter = ui.painter();
    let card_fill = if response.hovered() { egui::Color32::from_rgb(37, 38, 42) } else { egui::Color32::from_rgb(31, 32, 36) };
    painter.rect_filled(rect, 9.0, card_fill);
    painter.rect_stroke(
        rect,
        9.0,
        egui::Stroke::new(1.0, if response.hovered() { egui::Color32::from_rgb(111, 61, 64) } else { egui::Color32::from_rgb(55, 56, 60) }),
        egui::StrokeKind::Inside,
    );
    let center = egui::pos2(rect.center().x, rect.top() + 83.0);
    let radius = 38.0;
    let range = param.range();
    let (min_value, max_value) = match range {
        FloatRange::Linear { min, max } => (min, max),
        FloatRange::Skewed { min, max, .. } => (min, max),
        FloatRange::SymmetricalSkewed { min, max, .. } => (min, max),
        FloatRange::Reversed(inner) => {
            match *inner {
                FloatRange::Linear { min, max } => (min, max),
                FloatRange::Skewed { min, max, .. } => (min, max),
                FloatRange::SymmetricalSkewed { min, max, .. } => (min, max),
                FloatRange::Reversed(_) => unreachable!("Nested reversed FloatRange"),
            }
        }
    };
    let mut value = param.value();

    if response.dragged() {
        let delta = -response.drag_motion().y * (max_value - min_value) / 180.0;
        let new_value = (value + delta).clamp(min_value, max_value);
        setter.set_parameter(param, new_value);
        value = new_value;
    }

    let normalized = ((value - min_value) / (max_value - min_value)).clamp(0.0, 1.0);
    painter.circle_filled(center, radius + 8.0, egui::Color32::from_rgb(20, 21, 24));
    painter.circle_stroke(center, radius + 7.0, egui::Stroke::new(1.0, egui::Color32::from_rgb(69, 70, 74)));
    painter.circle_filled(center, radius, egui::Color32::from_rgb(48, 49, 53));
    painter.circle_stroke(center, radius, egui::Stroke::new(1.0, egui::Color32::from_rgb(87, 88, 92)));
    painter.circle_filled(center, radius - 6.0, egui::Color32::from_rgb(39, 40, 44));

    let start_angle = std::f32::consts::PI * 0.75;
    let end_angle = std::f32::consts::PI * 2.25;
    let angle = start_angle + normalized * (end_angle - start_angle);

    let mut arc_points = Vec::new();
    for i in 0..=48 {
        let t = i as f32 / 48.0;
        let a = start_angle + t * (end_angle - start_angle);
        arc_points.push(egui::pos2(center.x + (radius + 13.0) * a.cos(), center.y + (radius + 13.0) * a.sin()));
    }
    painter.add(egui::Shape::line(arc_points, egui::Stroke::new(2.0, egui::Color32::from_rgb(73, 49, 51))));
    for tick in 0..=10 {
        let t = tick as f32 / 10.0;
        let a = start_angle + t * (end_angle - start_angle);
        let inner = radius + 8.0;
        let outer = radius + if tick % 5 == 0 { 13.0 } else { 11.0 };
        painter.line_segment(
            [egui::pos2(center.x + inner * a.cos(), center.y + inner * a.sin()), egui::pos2(center.x + outer * a.cos(), center.y + outer * a.sin())],
            egui::Stroke::new(1.0, egui::Color32::from_rgb(145, 137, 135)),
        );
    }

    let mut active_points = Vec::new();
    let active_steps = (normalized * 48.0).round() as usize;
    for i in 0..=active_steps {
        let t = i as f32 / 48.0;
        let a = start_angle + t * (end_angle - start_angle);
        active_points.push(egui::pos2(center.x + (radius + 9.0) * a.cos(), center.y + (radius + 9.0) * a.sin()));
    }
    if active_points.len() > 1 {
        painter.add(egui::Shape::line(active_points, egui::Stroke::new(4.0, accent)));
    }

    let pointer_end = egui::pos2(center.x + (radius - 9.0) * angle.cos(), center.y + (radius - 9.0) * angle.sin());
    painter.line_segment([center, pointer_end], egui::Stroke::new(2.5, accent));
    painter.circle_filled(center, 4.0, egui::Color32::from_rgb(211, 208, 204));

    let value_text = if label == "OUTPUT" {
        format!("{value:.1} dB")
    } else {
        format!("{value:.0}%")
    };
    painter.text(
        egui::pos2(rect.center().x, rect.top() + 151.0),
        egui::Align2::CENTER_CENTER,
        value_text,
        egui::FontId::proportional(19.0),
        egui::Color32::from_rgb(237, 235, 231),
    );
    painter.text(
        egui::pos2(rect.center().x, rect.top() + 180.0),
        egui::Align2::CENTER_CENTER,
        label,
        egui::FontId::proportional(11.0),
        egui::Color32::from_rgb(208, 204, 199),
    );
    painter.text(
        egui::pos2(rect.center().x, rect.top() + 199.0),
        egui::Align2::CENTER_CENTER,
        descriptor,
        egui::FontId::proportional(8.0),
        egui::Color32::from_rgb(126, 128, 132),
    );

    if response.double_clicked() {
        setter.set_parameter(param, param.default_plain_value());
    }
}

impl Vst3Plugin for EmberFat {
    const VST3_CLASS_ID: [u8; 16] = *b"EmberFatPlugin01";

    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Distortion];
}

nih_export_vst3!(EmberFat);

