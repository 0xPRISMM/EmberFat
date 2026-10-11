use nih_plug::prelude::*;
use nih_plug_egui::EguiState;

mod dsp;
mod editor;
use std::sync::Arc;

struct EmberFat {
    params: Arc<EmberFatParams>,
    dsp: dsp::Dsp,
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
                    max: 24.0,
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
        let params = Arc::new(EmberFatParams::default());
        Self {
            dsp: dsp::Dsp::new(48000.0, params.controls()),
            params,
        }
    }
}

impl EmberFatParams {
    fn controls(&self) -> dsp::Controls {
        dsp::Controls {
            fat: f64::from(self.fat.value()) / 100.0,
            color: f64::from(self.color.value()) / 100.0,
            mix: f64::from(self.mix.value()) / 100.0,
            gain: f64::from(util::db_to_gain(self.output.value())),
            bypass: self.bypass.value(),
        }
    }
}

impl Plugin for EmberFat {
    const NAME: &'static str = "EmberFat";
    const VENDOR: &'static str = "eM4";
    const URL: &'static str = "https://github.com/0xPRISMM/EmberFat";
    const EMAIL: &'static str = "";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

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
        buffer_config: &BufferConfig,
        context: &mut impl InitContext<Self>,
    ) -> bool {
        self.dsp = dsp::Dsp::new(f64::from(buffer_config.sample_rate), self.params.controls());
        context.set_latency_samples(dsp::LATENCY as u32);
        true
    }

    fn reset(&mut self) {
        self.dsp.reset(self.params.controls());
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        self.dsp.set_controls(self.params.controls());
        for channel_samples in buffer.iter_samples() {
            self.dsp.frame(channel_samples.into_iter());
        }

        ProcessStatus::Normal
    }
}

impl Vst3Plugin for EmberFat {
    const VST3_CLASS_ID: [u8; 16] = *b"EmberFatPlugin01";

    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[Vst3SubCategory::Fx];
}

nih_export_vst3!(EmberFat);
