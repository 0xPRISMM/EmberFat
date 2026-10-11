use super::*;

fn controls() -> Controls {
    Controls {
        fat: 0.32,
        color: 0.28,
        mix: 1.0,
        gain: 1.0,
        bypass: false,
    }
}

fn tick(dsp: &mut Dsp, left: f32, right: f32) -> [f32; 2] {
    let mut samples = [left, right];
    dsp.frame(samples.iter_mut());
    samples
}

#[test]
fn dry_modes_preserve_headroom_and_exact_latency() {
    for settings in [
        Controls {
            mix: 0.0,
            ..controls()
        },
        Controls {
            bypass: true,
            ..controls()
        },
        Controls {
            fat: 0.0,
            ..controls()
        },
    ] {
        let mut dsp = Dsp::new(48000.0, settings);
        for n in 0..256 {
            let input = if n == 0 { 3.0 } else { 0.0 };
            let expected = if n == LATENCY { 3.0 } else { 0.0 };
            assert_eq!(tick(&mut dsp, input, -input), [expected, -expected]);
        }
    }
}

#[test]
fn oversampling_latency_matches_the_dry_delay() {
    let kernel = Kernel::default();
    let mut up1 = Fir::default();
    let mut up2 = Fir::default();
    let mut down2 = Fir::default();
    let mut down1 = Fir::default();
    let mut output = vec![];
    for n in 0..256 {
        let up = [
            up1.tick(if n == 0 { 2.0 } else { 0.0 }, &kernel),
            up1.tick(0.0, &kernel),
        ];
        let mut down = [0.0; 2];
        for (i, x) in up.into_iter().enumerate() {
            down[i] = down2.tick(up2.tick(x * 2.0, &kernel), &kernel);
            down2.tick(up2.tick(0.0, &kernel), &kernel);
        }
        output.push(down1.tick(down[0], &kernel));
        down1.tick(down[1], &kernel);
    }
    let peak = output
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.abs().total_cmp(&b.1.abs()))
        .unwrap()
        .0;
    assert_eq!(peak, LATENCY);
    assert!((output.iter().sum::<f64>() - 1.0).abs() < 1e-7);
    for i in 0..LATENCY {
        assert!((output[i] - output[2 * LATENCY - i]).abs() < 1e-14);
    }
}

#[test]
fn control_and_bypass_changes_have_no_block_edge_jump() {
    for rate in [44100.0, 48000.0, 96000.0] {
        for updated in [
            Controls {
                fat: 1.0,
                ..controls()
            },
            Controls {
                color: 1.0,
                ..controls()
            },
            Controls {
                mix: 0.0,
                ..controls()
            },
            Controls {
                gain: 2.0,
                ..controls()
            },
            Controls {
                bypass: true,
                ..controls()
            },
        ] {
            let mut dsp = Dsp::new(rate, controls());
            let mut previous = 0.0;
            for _ in 0..rate as usize {
                previous = tick(&mut dsp, 0.2, 0.2)[0];
            }
            dsp.set_controls(updated);
            let mut largest = 0.0_f32;
            for _ in 0..(rate * 0.1) as usize {
                let output = tick(&mut dsp, 0.2, 0.2)[0];
                largest = largest.max((output - previous).abs());
                previous = output;
            }
            assert!(largest < 0.002, "rate={rate}, change={largest}");
        }
    }
}

#[test]
fn silence_is_zero_and_filters_recover_from_invalid_input() {
    let mut dsp = Dsp::new(
        48000.0,
        Controls {
            fat: 1.0,
            color: 1.0,
            ..controls()
        },
    );
    for _ in 0..8192 {
        assert_eq!(tick(&mut dsp, 0.0, 0.0), [0.0; 2]);
    }
    for x in [
        f32::NAN,
        f32::INFINITY,
        f32::NEG_INFINITY,
        f32::MAX,
        -f32::MAX,
        64.0,
        -64.0,
    ] {
        assert!(tick(&mut dsp, x, -x).iter().all(|v| v.is_finite()));
    }
    let mut last = [0.0; 2];
    for _ in 0..48000 {
        last = tick(&mut dsp, 0.0, 0.0);
    }
    assert!(last.iter().all(|v| v.abs() < 1e-10));
}

fn render_blocks(size: usize) -> Vec<f32> {
    let mut dsp = Dsp::new(48000.0, controls());
    let mut output = vec![];
    for event in 0..8 {
        let settings = Controls {
            fat: event as f64 / 7.0,
            gain: 0.5 + event as f64 * 0.2,
            bypass: event == 5,
            ..controls()
        };
        for block in (0..4096).step_by(size) {
            dsp.set_controls(settings);
            for n in block..(block + size).min(4096) {
                let x = (0.4 * (TAU * 997.0 * (event * 4096 + n) as f64 / 48000.0).sin()) as f32;
                let frame = tick(&mut dsp, x, x);
                assert_eq!(frame[0], frame[1]);
                output.push(frame[0]);
            }
        }
    }
    output
}

#[test]
fn automation_is_independent_of_block_size_and_stereo_channel() {
    let reference = render_blocks(1);
    for size in [7, 32, 64, 128, 512] {
        assert_eq!(render_blocks(size), reference);
    }
}

#[test]
fn reset_clears_tails_and_channels_do_not_leak() {
    let mut dsp = Dsp::new(48000.0, controls());
    for n in 0..8192 {
        let output = tick(&mut dsp, if n % 111 == 0 { 0.7 } else { 0.0 }, 0.0);
        assert_eq!(output[1], 0.0);
    }
    dsp.reset(controls());
    let mut fresh = Dsp::new(48000.0, controls());
    for n in 0..2048 {
        let x = (n as f32 * 0.15).sin() * 0.3;
        assert_eq!(tick(&mut dsp, x, -x), tick(&mut fresh, x, -x));
    }
}

#[test]
fn asymmetric_color_generates_even_harmonics_without_dc() {
    let mut dsp = Dsp::new(
        48000.0,
        Controls {
            fat: 0.8,
            color: 1.0,
            ..controls()
        },
    );
    let mut sum = 0.0;
    let mut even_re = 0.0;
    let mut even_im = 0.0;
    for n in 0..96000 {
        let phase = TAU * 1000.0 * n as f64 / 48000.0;
        let y = f64::from(tick(&mut dsp, (0.5 * phase.sin()) as f32, 0.0)[0]);
        if n >= 48000 {
            sum += y;
            even_re += y * (2.0 * phase).cos();
            even_im += y * (2.0 * phase).sin();
        }
    }
    assert!((sum / 48000.0).abs() < 1e-7);
    assert!(2.0 * even_re.hypot(even_im) / 48000.0 > 0.001);
}

// Explicitly requested export/benchmark; ordinary test runs do not write artifacts.
#[test]
fn export_audio_validation_fixtures() {
    let Some(directory) = std::env::var_os("EMBERFAT_AUDIO_TEST_DIR") else {
        return;
    };
    use std::io::Write;
    let path = std::path::Path::new(&directory);
    std::fs::create_dir_all(path).unwrap();
    for (name, settings, frequency) in [
        (
            "alias",
            Controls {
                fat: 0.8,
                color: 0.4,
                ..controls()
            },
            9000.0,
        ),
        ("automation", controls(), 0.0),
    ] {
        let mut dsp = Dsp::new(48000.0, settings);
        let mut file = std::fs::File::create(path.join(format!("{name}.f32"))).unwrap();
        let mut raw = std::fs::File::create(path.join(format!("{name}-1x.f64"))).unwrap();
        let mut raw_dc = 0.0;
        let mut raw_prev = 0.0;
        let mut raw_tone = 0.0;
        for n in 0..96000 {
            let actual = if name == "automation" && n >= 48000 {
                Controls {
                    fat: 1.0,
                    color: 1.0,
                    gain: 2.0,
                    ..settings
                }
            } else {
                settings
            };
            dsp.set_controls(actual);
            let x = if frequency == 0.0 {
                0.2
            } else {
                0.6 * (TAU * frequency * n as f64 / 48000.0).sin()
            };
            let y = tick(&mut dsp, x as f32, x as f32)[0];
            file.write_all(&y.to_le_bytes()).unwrap();
            let residual = Shape::new(actual.fat, actual.color).residual(x);
            raw_dc = residual - raw_prev + dsp.dc_pole * raw_dc;
            raw_prev = residual;
            let pole = dsp.bright_pole + actual.color * (dsp.dark_pole - dsp.bright_pole);
            raw_tone = (1.0 - pole) * raw_dc + pole * raw_tone;
            raw.write_all(&((x + raw_tone) * actual.gain).to_le_bytes())
                .unwrap();
        }
    }
    let mut dsp = Dsp::new(48000.0, controls());
    let start = std::time::Instant::now();
    let mut checksum = 0.0;
    for n in 0..480000 {
        let x = ((n as f64 * 0.15).sin() * 0.3) as f32;
        checksum += std::hint::black_box(tick(&mut dsp, x, -x)[0]);
    }
    let elapsed = start.elapsed().as_secs_f64();
    println!("DSP_BENCH: 10 seconds stereo / {elapsed:.4} seconds CPU; checksum={checksum}");
    std::fs::write(path.join("benchmark.txt"), format!("{elapsed:.6}\n")).unwrap();
}
