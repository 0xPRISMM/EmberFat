use super::*;

#[test]
fn oversampling_reduces_folded_harmonics_for_the_same_curve() {
    let fat = 0.8;
    let mut compressor = Compressor::new(48000.0);
    let mut gain = 1.0;
    for _ in 0..48000 {
        gain = compressor.gain([0.6; 2], fat);
    }
    let mut shape = Shape::new(fat, 0.0);
    shape.drive *= gain;
    let kernel = Kernel::default();
    let mut channel = Channel::default();
    let dc_pole = (-TAU * 10.0 / 48000.0).exp();
    let tone_pole = (-TAU * 8000.0 / 48000.0).exp();
    let mut previous = 0.0;
    let mut dc = 0.0;
    let mut tone = 0.0;
    let mut projections = [[(0.0, 0.0); 5]; 2];
    // Same steady compressor gain and nonlinear curve in both paths,
    // isolating anti-aliasing from envelope and limiter modulation.
    for n in 0..96000 {
        let input = (0.6 * (TAU * 9000.0 * n as f64 / 48000.0).sin()) as f32;
        let (_, filtered) = channel.tick(
            f64::from(input),
            &shape,
            &kernel,
            dc_pole,
            tone_pole,
            0.0,
            0.0,
        );
        let residual = shape.residual(f64::from(input));
        dc = residual - previous + dc_pole * dc;
        previous = residual;
        tone = (1.0 - tone_pole) * dc + tone_pole * tone;
        if n >= 48000 {
            for (bin, frequency) in [3000.0, 6000.0, 12000.0, 15000.0, 21000.0]
                .into_iter()
                .enumerate()
            {
                let phase = TAU * frequency * n as f64 / 48000.0;
                for (version, value) in [filtered, f64::from(input) + tone].into_iter().enumerate()
                {
                    projections[version][bin].0 += value * phase.cos();
                    projections[version][bin].1 += value * phase.sin();
                }
            }
        }
    }
    let energy = projections.map(|bins| bins.iter().map(|(re, im)| re * re + im * im).sum::<f64>());
    let improvement = 10.0 * (energy[1] / energy[0]).log10();
    println!("ALIAS_REDUCTION_DB: {improvement:.2}");
    assert!(improvement > 30.0, "Alias reduction={improvement}");
}
