//! EmberFat's original saturation engine. No allocation, locks, or host calls in processing.
use std::f64::consts::{PI, TAU};

const OVERSAMPLING_LATENCY: usize = 56;
const LOOKAHEAD: usize = 24;
pub const LATENCY: usize = OVERSAMPLING_LATENCY + LOOKAHEAD;

#[derive(Clone, Copy)]
pub struct Controls {
    pub fat: f64,
    pub color: f64,
    pub mix: f64,
    pub gain: f64,
    pub bypass: bool,
}

#[derive(Clone, Copy, Default)]
struct Ramp {
    value: f64,
    target: f64,
    step: f64,
    remaining: usize,
}

impl Ramp {
    fn reset(&mut self, value: f64) {
        *self = Self {
            value,
            target: value,
            ..Self::default()
        };
    }

    fn target(&mut self, target: f64, samples: usize) {
        // Do not restart a ramp at every host block; its duration is in samples.
        if target != self.target {
            self.target = target;
            self.remaining = samples.max(1);
            self.step = (target - self.value) / self.remaining as f64;
        }
    }

    fn next(&mut self) -> f64 {
        if self.remaining != 0 {
            self.value += self.step;
            self.remaining -= 1;
            if self.remaining == 0 {
                self.value = self.target;
            }
        }
        self.value
    }
}

// A 65-tap Blackman-windowed half-band FIR, designed here from the sinc equation.
// Symmetric tap pairs avoid multiplying the zero coefficients. Each 2x stage
// delays by 16 samples at its lower rate; three up/down stages total 56 base samples.
struct Kernel {
    pairs: [f64; 16],
    center: f64,
}

impl Default for Kernel {
    fn default() -> Self {
        let mut pairs = [0.0; 16];
        for (pair, coefficient) in pairs.iter_mut().enumerate() {
            let index = (pair * 2 + 1) as f64;
            let distance = index - 32.0;
            let window =
                0.42 - 0.5 * (TAU * index / 64.0).cos() + 0.08 * (2.0 * TAU * index / 64.0).cos();
            *coefficient = (PI * 0.5 * distance).sin() / (PI * distance) * window;
        }
        let normalization = 0.5 + 2.0 * pairs.iter().sum::<f64>();
        for coefficient in &mut pairs {
            *coefficient /= normalization;
        }
        Self {
            pairs,
            center: 0.5 / normalization,
        }
    }
}

#[derive(Clone)]
struct Fir {
    history: [f64; 128],
    position: usize,
}

impl Default for Fir {
    fn default() -> Self {
        Self {
            history: [0.0; 128],
            position: 0,
        }
    }
}

impl Fir {
    fn tick(&mut self, input: f64, kernel: &Kernel) -> f64 {
        let p = self.position;
        self.history[p] = input;
        let mut result = kernel.center * self.history[p.wrapping_sub(32) & 127];
        for (pair, coefficient) in kernel.pairs.iter().enumerate() {
            let offset = pair * 2 + 1;
            result += coefficient
                * (self.history[p.wrapping_sub(offset) & 127]
                    + self.history[p.wrapping_sub(64 - offset) & 127]);
        }
        self.position = (p + 1) & 127;
        result
    }
}

#[derive(Clone)]
struct Channel {
    up1: Fir,
    up2: Fir,
    up3: Fir,
    down3: Fir,
    down2: Fir,
    down1: Fir,
    dry: [f64; OVERSAMPLING_LATENCY],
    dry_position: usize,
    previous_residual: f64,
    dc: f64,
    tone: f64,
    emphasis_low: f64,
}

impl Default for Channel {
    fn default() -> Self {
        Self {
            up1: Fir::default(),
            up2: Fir::default(),
            up3: Fir::default(),
            down3: Fir::default(),
            down2: Fir::default(),
            down1: Fir::default(),
            dry: [0.0; OVERSAMPLING_LATENCY],
            dry_position: 0,
            previous_residual: 0.0,
            dc: 0.0,
            tone: 0.0,
            emphasis_low: 0.0,
        }
    }
}

struct Shape {
    drive: f64,
    bias: f64,
    zero: f64,
    rounded_zero: f64,
    hardness: f64,
    density: f64,
    emphasis: f64,
}

fn rounded_clip(x: f64) -> f64 {
    let x = x.clamp(-1.0, 1.0);
    x * (1.5 - 0.5 * x * x)
}

impl Shape {
    fn new(fat: f64, color: f64) -> Self {
        let bias = 0.12 * color * fat;
        Self {
            // Do not normalize away the drive: FAT is meant to raise density
            // and loudness. The linked compressor controls its dynamics.
            drive: (1.0 + 3.0 * fat + 24.0 * fat * fat) * 10.0_f64.powf(10.0 * fat / 20.0),
            bias,
            zero: bias.tanh(),
            rounded_zero: rounded_clip(bias),
            hardness: fat * fat * (0.25 + 0.75 * color),
            density: fat * (2.0 - fat),
            emphasis: 1.5 * color * fat,
        }
    }

    fn residual(&self, input: f64) -> f64 {
        let driven = self.drive * input + self.bias;
        let soft = driven.tanh() - self.zero;
        let rounded = rounded_clip(driven) - self.rounded_zero;
        self.density * (soft + self.hardness * (rounded - soft) - input)
    }
}

struct Compressor {
    envelope: f64,
    attack: f64,
    release: f64,
}

impl Compressor {
    fn new(rate: f64) -> Self {
        Self {
            envelope: 0.0,
            attack: (-1.0 / (0.002 * rate)).exp(),
            release: (-1.0 / (0.080 * rate)).exp(),
        }
    }

    fn gain(&mut self, inputs: [f64; 2], fat: f64) -> f64 {
        // The louder channel controls both: compression never moves the stereo image.
        let peak = inputs[0].abs().max(inputs[1].abs()).min(64.0);
        let pole = if peak > self.envelope {
            self.attack
        } else {
            self.release
        };
        self.envelope = peak + pole * (self.envelope - peak);
        if self.envelope < 1e-30 {
            self.envelope = 0.0;
        }
        let level = 20.0 * self.envelope.max(1e-12).log10();
        let over = level - (-10.0 - 18.0 * fat);
        let knee = 6.0;
        let reduction = if over <= -knee / 2.0 {
            0.0
        } else if over >= knee / 2.0 {
            over
        } else {
            (over + knee / 2.0).powi(2) / (2.0 * knee)
        };
        let ratio = 1.0 + 11.0 * fat;
        10.0_f64.powf(-reduction * (1.0 - 1.0 / ratio) / 20.0)
    }
}

struct PeakControl {
    dry: [[f64; 2]; LOOKAHEAD],
    wet: [[f64; 2]; LOOKAHEAD],
    position: usize,
    gain: f64,
    attack: f64,
    release: f64,
}

impl PeakControl {
    fn new(rate: f64) -> Self {
        Self {
            dry: [[0.0; 2]; LOOKAHEAD],
            wet: [[0.0; 2]; LOOKAHEAD],
            position: 0,
            gain: 1.0,
            attack: (-1.0 / (0.0001 * rate)).exp(),
            release: (-1.0 / (0.060 * rate)).exp(),
        }
    }

    fn tick(&mut self, dry: [f64; 2], wet: [f64; 2], density: f64) -> ([f64; 2], [f64; 2]) {
        let delayed_dry = self.dry[self.position];
        let delayed_wet = self.wet[self.position];
        self.dry[self.position] = dry;
        self.wet[self.position] = wet;
        self.position = (self.position + 1) % LOOKAHEAD;
        // Peak control blends in with density: clean at FAT=0, +/-1 at FAT=100. Keep a
        // fixed delay even when inactive, so mix/bypass/automation never comb-filter.
        let ceiling = 1.0 / density.max(1e-30);
        let current_peak = delayed_wet[0].abs().max(delayed_wet[1].abs());
        let peak = self.wet.iter().fold(current_peak, |p, frame| {
            p.max(frame[0].abs()).max(frame[1].abs())
        });
        let required = (ceiling / peak.max(ceiling)).min(1.0);
        let pole = if required < self.gain {
            self.attack
        } else {
            self.release
        };
        self.gain = required + pole * (self.gain - required);
        // Final linked gain bound guarantees the maximum-FAT ceiling despite release
        // and control changes. OUTPUT remains after peak control and can boost it.
        self.gain = self.gain.min(ceiling / current_peak.max(ceiling));
        (
            delayed_dry,
            delayed_wet.map(|x| x * (1.0 - density + density * self.gain)),
        )
    }
}

impl Channel {
    fn tick(
        &mut self,
        input: f64,
        shape: &Shape,
        kernel: &Kernel,
        dc_pole: f64,
        tone_pole: f64,
        emphasis_pole: f64,
        color: f64,
    ) -> (f64, f64) {
        let dry = self.dry[self.dry_position];
        self.dry[self.dry_position] = input;
        self.dry_position = (self.dry_position + 1) % OVERSAMPLING_LATENCY;
        let safe_input = input.clamp(-64.0, 64.0);
        let up = [
            self.up1.tick(safe_input * 2.0, kernel),
            self.up1.tick(0.0, kernel),
        ];
        let mut down = [0.0; 2];
        for (i, value) in up.into_iter().enumerate() {
            let middle = [
                self.up2.tick(value * 2.0, kernel),
                self.up2.tick(0.0, kernel),
            ];
            let mut reduced = [0.0; 2];
            for (j, value) in middle.into_iter().enumerate() {
                let high = [
                    self.up3.tick(value * 2.0, kernel),
                    self.up3.tick(0.0, kernel),
                ];
                let mut residual = [0.0; 2];
                for (k, x) in high.into_iter().enumerate() {
                    self.emphasis_low = x + emphasis_pole * (self.emphasis_low - x);
                    let colored = x + shape.emphasis * (x - self.emphasis_low);
                    // Subtract the original interpolation, not the emphasized
                    // input, so COLOR's added brightness remains in the wet path.
                    residual[k] = shape.residual(colored) + shape.density * (colored - x);
                }
                reduced[j] = self.down3.tick(residual[0], kernel);
                self.down3.tick(residual[1], kernel);
            }
            down[i] = self.down2.tick(reduced[0], kernel);
            self.down2.tick(reduced[1], kernel);
        }
        let residual = self.down1.tick(down[0], kernel);
        self.down1.tick(down[1], kernel);
        self.dc = residual - self.previous_residual + dc_pole * self.dc;
        self.previous_residual = residual;
        self.tone = (1.0 - tone_pole) * self.dc + tone_pole * self.tone;
        if self.dc.abs() < 1e-30 {
            self.dc = 0.0;
        }
        if self.tone.abs() < 1e-30 {
            self.tone = 0.0;
        }
        if self.emphasis_low.abs() < 1e-30 {
            self.emphasis_low = 0.0;
        }
        (dry, dry + self.tone + color * (self.dc - self.tone))
    }
}

pub struct Dsp {
    channels: [Channel; 2],
    kernel: Kernel,
    compressor: Compressor,
    peaks: PeakControl,
    fat: Ramp,
    color: Ramp,
    mix: Ramp,
    gain: Ramp,
    active: Ramp,
    control_samples: usize,
    bypass_samples: usize,
    dc_pole: f64,
    warm_pole: f64,
    emphasis_pole: f64,
}

impl Dsp {
    pub fn new(sample_rate: f64, controls: Controls) -> Self {
        let mut result = Self {
            channels: std::array::from_fn(|_| Channel::default()),
            kernel: Kernel::default(),
            compressor: Compressor::new(sample_rate),
            peaks: PeakControl::new(sample_rate),
            fat: Ramp::default(),
            color: Ramp::default(),
            mix: Ramp::default(),
            gain: Ramp::default(),
            active: Ramp::default(),
            control_samples: (sample_rate * 0.020).round().max(1.0) as usize,
            bypass_samples: (sample_rate * 0.005).round().max(1.0) as usize,
            dc_pole: (-TAU * 10.0 / sample_rate).exp(),
            warm_pole: (-TAU * 8000.0 / sample_rate).exp(),
            emphasis_pole: (-TAU * 2500.0 / (sample_rate * 8.0)).exp(),
        };
        result.reset(controls);
        result
    }

    pub fn reset(&mut self, controls: Controls) {
        for channel in &mut self.channels {
            *channel = Channel::default();
        }
        self.compressor.envelope = 0.0;
        self.peaks.dry.fill([0.0; 2]);
        self.peaks.wet.fill([0.0; 2]);
        self.peaks.position = 0;
        self.peaks.gain = 1.0;
        self.fat.reset(controls.fat);
        self.color.reset(controls.color);
        self.mix.reset(controls.mix);
        self.gain.reset(controls.gain);
        self.active.reset(if controls.bypass { 0.0 } else { 1.0 });
    }

    pub fn set_controls(&mut self, controls: Controls) {
        self.fat.target(controls.fat, self.control_samples);
        self.color.target(controls.color, self.control_samples);
        self.mix.target(controls.mix, self.control_samples);
        self.gain.target(controls.gain, self.control_samples);
        self.active
            .target(if controls.bypass { 0.0 } else { 1.0 }, self.bypass_samples);
    }

    pub fn frame<'a>(&mut self, mut samples: impl Iterator<Item = &'a mut f32>) {
        let refs: [Option<&mut f32>; 2] = std::array::from_fn(|_| samples.next());
        let inputs: [f64; 2] = std::array::from_fn(|i| {
            let value = refs[i].as_deref().copied().unwrap_or(0.0);
            if value.is_finite() {
                f64::from(value)
            } else {
                0.0
            }
        });
        let fat = self.fat.next();
        let color = self.color.next();
        let blend = self.mix.next() * self.active.next();
        let gain = self.gain.next();
        let mut shape = Shape::new(fat, color);
        shape.drive *= self.compressor.gain(inputs, fat);
        let mut dry = [0.0; 2];
        let mut wet = [0.0; 2];
        for (i, channel) in self.channels.iter_mut().enumerate() {
            (dry[i], wet[i]) = channel.tick(
                inputs[i],
                &shape,
                &self.kernel,
                self.dc_pole,
                self.warm_pole,
                self.emphasis_pole,
                color,
            );
        }
        let (dry, wet) = self.peaks.tick(dry, wet, shape.density);
        for (i, sample) in refs.into_iter().enumerate() {
            if let Some(sample) = sample {
                let result = dry[i] + blend * (wet[i] * gain - dry[i]);
                *sample = result.clamp(-(f32::MAX as f64), f32::MAX as f64) as f32;
            }
        }
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod alias_tests;
