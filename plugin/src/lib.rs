use nih_plug::prelude::*;
use nih_plug_egui::EguiState;

mod editor;
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
                FloatRange::Linear {
                    min: 0.0,
                    max: 100.0,
                },
            )
            .with_unit("%")
            .with_step_size(0.1),
            color: FloatParam::new(
                "Color",
                28.0,
                FloatRange::Linear {
                    min: 0.0,
                    max: 100.0,
                },
            )
            .with_unit("%")
            .with_step_size(0.1),
            mix: FloatParam::new(
                "Mix",
                100.0,
                FloatRange::Linear {
                    min: 0.0,
                    max: 100.0,
                },
            )
            .with_unit("%")
            .with_step_size(0.1),
            output: FloatParam::new(
                "Output",
                -1.0,
                FloatRange::Linear {
                    min: -18.0,
                    max: 6.0,
                },
            )
            .with_unit(" dB")
            .with_step_size(0.1),
            bypass: BoolParam::new("Bypass", false),
        }
    }
}

impl Default for EmberFat {
    fn default() -> Self {
        Self {
            params: Arc::new(EmberFatParams::default()),
        }
    }
}

impl Plugin for EmberFat {
    const NAME: &'static str = "EmberFat";
    const VENDOR: &'static str = "Ember Audio Lab";
    const URL: &'static str = "https://github.com/0xPRISMM/EmberFat";
    const EMAIL: &'static str = "";
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
        editor::create(self.params.clone())
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

impl Vst3Plugin for EmberFat {
    const VST3_CLASS_ID: [u8; 16] = *b"EmberFatPlugin01";

    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Distortion];
}

nih_export_vst3!(EmberFat);
