//! EmberFat's original saturation engine. No allocation, locks, or host calls in processing.
use std::f64::consts::{PI, TAU};

pub const LATENCY: usize = 48;

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
// delays by 16 samples at its lower rate; two up/down stages total 48 base samples.
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
    down2: Fir,
    down1: Fir,
    dry: [f64; LATENCY],
    dry_position: usize,
    previous_residual: f64,
    dc: f64,
    tone: f64,
}

impl Default for Channel {
    fn default() -> Self {
        Self {
            up1: Fir::default(),
            up2: Fir::default(),
            down2: Fir::default(),
            down1: Fir::default(),
            dry: [0.0; LATENCY],
            dry_position: 0,
            previous_residual: 0.0,
            dc: 0.0,
            tone: 0.0,
        }
    }
}

struct Shape {
    drive: f64,
    bias: f64,
    zero: f64,
    compensation: f64,
    density: f64,
}

impl Shape {
    fn new(fat: f64, color: f64) -> Self {
        let drive = 1.0 + 7.0 * fat;
        let bias = 0.25 * color * fat;
        Self {
            drive,
            bias,
            zero: bias.tanh(),
            compensation: 1.0 / (1.0 + 0.6 * (drive - 1.0)),
            density: fat * (2.0 - fat),
        }
    }

    fn residual(&self, input: f64) -> f64 {
        // Bias generates genuine even harmonics; zero input still produces zero.
        // FAT=0 is exactly transparent. Density and compensation prevent the old
        // large small-signal boost while retaining a progressive soft knee.
        self.density
            * (((self.drive * input + self.bias).tanh() - self.zero) * self.compensation - input)
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
    ) -> (f64, f64) {
        let dry = self.dry[self.dry_position];
        self.dry[self.dry_position] = input;
        self.dry_position = (self.dry_position + 1) % LATENCY;

        let safe_input = input.clamp(-64.0, 64.0);
        let up = [
            self.up1.tick(safe_input * 2.0, kernel),
            self.up1.tick(0.0, kernel),
        ];
        let mut down = [0.0; 2];
        for (index, value) in up.into_iter().enumerate() {
            let first = self.up2.tick(value * 2.0, kernel);
            let second = self.up2.tick(0.0, kernel);
            down[index] = self.down2.tick(shape.residual(first), kernel);
            self.down2.tick(shape.residual(second), kernel);
        }
        let residual = self.down1.tick(down[0], kernel);
        self.down1.tick(down[1], kernel);

        // Filter only added distortion: the delayed clean signal keeps its bass,
        // phase and headroom. High-pass the asymmetric residual to remove DC.
        self.dc = residual - self.previous_residual + dc_pole * self.dc;
        self.previous_residual = residual;
        self.tone = (1.0 - tone_pole) * self.dc + tone_pole * self.tone;
        if self.dc.abs() < 1e-30 {
            self.dc = 0.0;
        }
        if self.tone.abs() < 1e-30 {
            self.tone = 0.0;
        }
        (dry, dry + self.tone)
    }
}

pub struct Dsp {
    channels: [Channel; 2],
    kernel: Kernel,
    fat: Ramp,
    color: Ramp,
    mix: Ramp,
    gain: Ramp,
    active: Ramp,
    control_samples: usize,
    bypass_samples: usize,
    dc_pole: f64,
    bright_pole: f64,
    dark_pole: f64,
}

impl Dsp {
    pub fn new(sample_rate: f64, controls: Controls) -> Self {
        let mut result = Self {
            channels: std::array::from_fn(|_| Channel::default()),
            kernel: Kernel::default(),
            fat: Ramp::default(),
            color: Ramp::default(),
            mix: Ramp::default(),
            gain: Ramp::default(),
            active: Ramp::default(),
            control_samples: (sample_rate * 0.020).round().max(1.0) as usize,
            bypass_samples: (sample_rate * 0.005).round().max(1.0) as usize,
            dc_pole: (-TAU * 10.0 / sample_rate).exp(),
            bright_pole: (-TAU * 18000.0 / sample_rate).exp(),
            dark_pole: (-TAU * 4000.0 / sample_rate).exp(),
        };
        result.reset(controls);
        result
    }

    pub fn reset(&mut self, controls: Controls) {
        for channel in &mut self.channels {
            *channel = Channel::default();
        }
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

    pub fn frame<'a>(&mut self, samples: impl Iterator<Item = &'a mut f32>) {
        // Shared ramps advance once per frame, keeping left/right automation identical.
        let fat = self.fat.next();
        let color = self.color.next();
        let blend = self.mix.next() * self.active.next();
        let gain = self.gain.next();
        let shape = Shape::new(fat, color);
        let tone_pole = self.bright_pole + color * (self.dark_pole - self.bright_pole);
        for (channel, sample) in self.channels.iter_mut().zip(samples) {
            let input = if sample.is_finite() {
                f64::from(*sample)
            } else {
                0.0
            };
            let (dry, wet) = channel.tick(input, &shape, &self.kernel, self.dc_pole, tone_pole);
            // Continue the filters during bypass/MIX=0 so re-enabling has no stale tail.
            let result = dry + blend * (wet * gain - dry);
            *sample = result.clamp(-(f32::MAX as f64), f32::MAX as f64) as f32;
        }
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod alias_tests;
