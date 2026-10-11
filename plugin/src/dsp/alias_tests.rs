use super::*;

#[test]
fn oversampling_reduces_folded_harmonics_for_the_same_curve() {
    let controls = Controls {
        fat: 0.8,
        color: 0.4,
        mix: 1.0,
        gain: 1.0,
        bypass: false,
    };
    let mut dsp = Dsp::new(48000.0, controls);
    let shape = Shape::new(controls.fat, controls.color);
    let pole = dsp.bright_pole + controls.color * (dsp.dark_pole - dsp.bright_pole);
    let mut previous = 0.0;
    let mut dc = 0.0;
    let mut tone = 0.0;
    let mut projections = [[(0.0, 0.0); 5]; 2];
    // Coherent 9 kHz input: 9 and 18 kHz are valid harmonics. The other
    // multiples of 3 kHz are harmonics folded back below Nyquist.
    for n in 0..96000 {
        let input = (0.6 * (TAU * 9000.0 * n as f64 / 48000.0).sin()) as f32;
        let mut samples = [input, input];
        dsp.frame(samples.iter_mut());
        let residual = shape.residual(f64::from(input));
        dc = residual - previous + dsp.dc_pole * dc;
        previous = residual;
        tone = (1.0 - pole) * dc + pole * tone;
        if n >= 48000 {
            for (bin, frequency) in [3000.0, 6000.0, 12000.0, 15000.0, 21000.0]
                .into_iter()
                .enumerate()
            {
                let phase = TAU * frequency * n as f64 / 48000.0;
                for (version, value) in [f64::from(samples[0]), f64::from(input) + tone]
                    .into_iter()
                    .enumerate()
                {
                    projections[version][bin].0 += value * phase.cos();
                    projections[version][bin].1 += value * phase.sin();
                }
            }
        }
    }
    let energy = projections.map(|bins| bins.iter().map(|(re, im)| re * re + im * im).sum::<f64>());
    assert!(
        energy[0] < energy[1] * 0.001,
        "Expected at least 30 dB alias reduction"
    );
}
