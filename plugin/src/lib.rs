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
            editor_state: EguiState::from_size(560, 340),
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
                ctx.set_style(style);
            },
            move |ctx, setter, _| {
                egui::CentralPanel::default()
                    .frame(egui::Frame::new().fill(egui::Color32::from_rgb(18, 18, 22)).inner_margin(egui::Margin::same(18)))
                    .show(ctx, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.label(
                                egui::RichText::new("EMBERFAT")
                                    .size(28.0)
                                    .strong()
                                    .color(egui::Color32::from_rgb(245, 70, 82)),
                            );
                            ui.label(
                                egui::RichText::new("HARMONIC SATURATION")
                                    .size(10.0)
                                    .color(egui::Color32::from_rgb(155, 155, 165)),
                            );
                        });
                        ui.add_space(18.0);

                        ui.horizontal_centered(|ui| {
                            draw_knob(ui, "FAT", &params.fat, setter, egui::Color32::from_rgb(240, 55, 70));
                            ui.add_space(10.0);
                            draw_knob(ui, "COLOR", &params.color, setter, egui::Color32::from_rgb(240, 75, 85));
                            ui.add_space(10.0);
                            draw_knob(ui, "MIX", &params.mix, setter, egui::Color32::from_rgb(210, 55, 70));
                            ui.add_space(10.0);
                            draw_knob(ui, "OUTPUT", &params.output, setter, egui::Color32::from_rgb(175, 45, 60));
                        });

                        ui.add_space(12.0);
                        ui.separator();
                        ui.horizontal(|ui| {
                            let mut bypass = params.bypass.value();
                            if ui.checkbox(&mut bypass, "BYPASS").changed() {
                                setter.set_parameter(&params.bypass, bypass);
                            }
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(egui::RichText::new("DRIVE • TONE • BLEND").size(10.0).color(egui::Color32::GRAY));
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
    param: &FloatParam,
    setter: &ParamSetter,
    accent: egui::Color32,
) {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(100.0, 142.0), egui::Sense::click_and_drag());
    let center = egui::pos2(rect.center().x, rect.top() + 48.0);
    let radius = 31.0;
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
    let painter = ui.painter();
    painter.circle_filled(center, radius + 5.0, egui::Color32::from_rgb(9, 9, 12));
    painter.circle_filled(center, radius, egui::Color32::from_rgb(42, 42, 49));
    painter.circle_stroke(center, radius, egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(76, 76, 84)));

    let start_angle = std::f32::consts::PI * 0.75;
    let end_angle = std::f32::consts::PI * 2.25;
    let angle = start_angle + normalized * (end_angle - start_angle);

    let mut arc_points = Vec::new();
    for i in 0..=48 {
        let t = i as f32 / 48.0;
        let a = start_angle + t * (end_angle - start_angle);
        arc_points.push(egui::pos2(center.x + (radius + 9.0) * a.cos(), center.y + (radius + 9.0) * a.sin()));
    }
    painter.add(egui::Shape::line(arc_points, egui::Stroke::new(3.0_f32, egui::Color32::from_rgb(65, 40, 45))));

    let mut active_points = Vec::new();
    let active_steps = (normalized * 48.0).round() as usize;
    for i in 0..=active_steps {
        let t = i as f32 / 48.0;
        let a = start_angle + t * (end_angle - start_angle);
        active_points.push(egui::pos2(center.x + (radius + 9.0) * a.cos(), center.y + (radius + 9.0) * a.sin()));
    }
    if active_points.len() > 1 {
        painter.add(egui::Shape::line(active_points, egui::Stroke::new(3.5_f32, accent)));
    }

    let pointer_end = egui::pos2(center.x + (radius - 8.0) * angle.cos(), center.y + (radius - 8.0) * angle.sin());
    painter.line_segment([center, pointer_end], egui::Stroke::new(3.0_f32, accent));
    painter.circle_filled(center, 4.0, egui::Color32::from_rgb(235, 235, 238));

    let value_text = if label == "OUTPUT" {
        format!("{value:.1} dB")
    } else {
        format!("{value:.0}%")
    };
    painter.text(
        egui::pos2(rect.center().x, rect.top() + 91.0),
        egui::Align2::CENTER_CENTER,
        value_text,
        egui::FontId::proportional(13.0),
        egui::Color32::from_rgb(230, 230, 235),
    );
    painter.text(
        egui::pos2(rect.center().x, rect.top() + 117.0),
        egui::Align2::CENTER_CENTER,
        label,
        egui::FontId::proportional(11.0),
        egui::Color32::from_rgb(170, 170, 180),
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
