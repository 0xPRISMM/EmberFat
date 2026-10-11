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
    let mut up = [Fir::default(), Fir::default(), Fir::default()];
    let mut down = [Fir::default(), Fir::default(), Fir::default()];
    let mut output = vec![];
    for n in 0..256 {
        let first = [
            up[0].tick(if n == 0 { 2.0 } else { 0.0 }, &kernel),
            up[0].tick(0.0, &kernel),
        ];
        let mut lower = [0.0; 2];
        for (i, x) in first.into_iter().enumerate() {
            let middle = [up[1].tick(x * 2.0, &kernel), up[1].tick(0.0, &kernel)];
            let mut upper = [0.0; 2];
            for (j, x) in middle.into_iter().enumerate() {
                upper[j] = down[2].tick(up[2].tick(x * 2.0, &kernel), &kernel);
                down[2].tick(up[2].tick(0.0, &kernel), &kernel);
            }
            lower[i] = down[1].tick(upper[0], &kernel);
            down[1].tick(upper[1], &kernel);
        }
        output.push(down[0].tick(lower[0], &kernel));
        down[0].tick(lower[1], &kernel);
    }
    let peak = output
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.abs().total_cmp(&b.1.abs()))
        .unwrap()
        .0;
    assert_eq!(peak, OVERSAMPLING_LATENCY);
    assert_eq!(LATENCY, OVERSAMPLING_LATENCY + LOOKAHEAD);
    assert!((output.iter().sum::<f64>() - 1.0).abs() < 1e-7);
    for i in 0..OVERSAMPLING_LATENCY {
        assert!((output[i] - output[2 * OVERSAMPLING_LATENCY - i]).abs() < 1e-14);
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

#[test]
fn compressor_reduces_dynamic_range_and_links_stereo() {
    for rate in [44100.0, 48000.0, 96000.0] {
        let mut c = Compressor::new(rate);
        let mut quiet = 1.0;
        for _ in 0..rate as usize {
            quiet = c.gain([0.02, 0.0], 1.0);
        }
        let mut loud = 1.0;
        for _ in 0..rate as usize {
            loud = c.gain([0.0, 0.8], 1.0);
        }
        assert!(quiet > 0.99);
        assert!(loud < 0.07, "loud gain={loud}");
        let linked = c.gain([0.8, 0.02], 1.0);
        let swapped = c.gain([0.02, 0.8], 1.0);
        assert!((linked - swapped).abs() < 1e-10);
        assert_eq!(c.gain([0.8, 0.8], 0.0), 1.0);
    }
}

#[test]
fn maximum_fat_peak_control_is_bounded_and_output_can_boost_it() {
    for rate in [44100.0, 48000.0, 96000.0] {
        let mut limiter = PeakControl::new(rate);
        for n in 0..4096 {
            let wet = [
                if n % 17 == 0 {
                    12.0
                } else {
                    (n as f64 * 0.17).sin() * 2.0
                },
                (n as f64 * 0.37).sin() * 3.0,
            ];
            let (_, limited) = limiter.tick([0.0; 2], wet, 1.0);
            assert!(limited.iter().all(|x| x.abs() <= 1.0 + 1e-12));
        }
    }
    let settings = Controls {
        fat: 1.0,
        color: 1.0,
        gain: 1.0,
        ..controls()
    };
    let mut unity = Dsp::new(48000.0, settings);
    let mut boosted = Dsp::new(
        48000.0,
        Controls {
            gain: 10.0_f64.powf(24.0 / 20.0),
            ..settings
        },
    );
    let mut peak = 0.0_f32;
    for n in 0..96000 {
        let x = (0.7 * (TAU * 97.0 * n as f64 / 48000.0).sin()) as f32;
        let a = tick(&mut unity, x, x)[0];
        let b = tick(&mut boosted, x, x)[0];
        assert!(a.abs() <= 1.000001);
        assert!(
            (f64::from(b) - f64::from(a) * settings.gain * 10.0_f64.powf(24.0 / 20.0)).abs() < 2e-6
        );
        peak = peak.max(b.abs());
    }
    assert!(
        peak > 10.0,
        "OUTPUT boost must not be undone by internal limiting"
    );
}

#[test]
fn fat_increases_loudness_and_density_on_quiet_bass() {
    let mut rms = [0.0_f64; 4];
    let mut crest = [0.0_f64; 4];
    for (i, fat) in [0.0, 0.25, 0.5, 1.0].into_iter().enumerate() {
        let mut dsp = Dsp::new(
            48000.0,
            Controls {
                fat,
                color: 0.28,
                ..controls()
            },
        );
        let mut peak = 0.0_f64;
        for n in 0..96000 {
            let x = (0.03 * (TAU * 100.0 * n as f64 / 48000.0).sin()) as f32;
            let y = f64::from(tick(&mut dsp, x, x)[0]);
            if n >= 48000 {
                rms[i] += y * y;
                peak = peak.max(y.abs());
            }
        }
        rms[i] = (rms[i] / 48000.0).sqrt();
        crest[i] = peak / rms[i];
    }
    assert!(rms.windows(2).all(|x| x[1] > x[0]), "RMS {rms:?}");
    assert!(20.0 * (rms[3] / rms[0]).log10() > 25.0, "RMS {rms:?}");
    assert!(crest[3] < crest[0] * 0.92, "crest {crest:?}");
    println!("INTENSITY: quiet bass RMS {rms:?}; crest {crest:?}");
}

#[test]
fn output_twenty_four_db_ramp_is_continuous() {
    for rate in [44100.0, 48000.0, 96000.0] {
        let settings = Controls {
            fat: 0.0,
            ..controls()
        };
        let mut dsp = Dsp::new(rate, settings);
        let mut previous = 0.0;
        for _ in 0..1024 {
            previous = tick(&mut dsp, 0.2, 0.2)[0];
        }
        let gain = 10.0_f64.powf(24.0 / 20.0);
        dsp.set_controls(Controls { gain, ..settings });
        let bound = 0.2 * (gain - 1.0) / (rate * 0.020).round();
        for _ in 0..(rate * 0.030) as usize {
            let next = tick(&mut dsp, 0.2, 0.2)[0];
            assert!(f64::from((next - previous).abs()) <= bound * 1.001);
            previous = next;
        }
        assert!((f64::from(previous) - 0.2 * gain).abs() < 1e-6);
    }
}

#[test]
fn color_increases_brightness_instead_of_darkening() {
    let mut amplitude = [[0.0_f64; 2]; 2];
    for (i, color) in [0.0, 1.0].into_iter().enumerate() {
        for (j, frequency) in [600.0, 6000.0].into_iter().enumerate() {
            let mut dsp = Dsp::new(
                48000.0,
                Controls {
                    fat: 0.3,
                    color,
                    ..controls()
                },
            );
            let mut re = 0.0;
            let mut im = 0.0;
            for n in 0..96000 {
                let phase = TAU * frequency * n as f64 / 48000.0;
                let y = f64::from(tick(&mut dsp, (0.015 * phase.sin()) as f32, 0.0)[0]);
                if n >= 48000 {
                    re += y * phase.cos();
                    im += y * phase.sin();
                }
            }
            amplitude[i][j] = 2.0 * re.hypot(im) / 48000.0;
        }
    }
    let warmth = amplitude[0][1] / amplitude[0][0];
    let brightness = amplitude[1][1] / amplitude[1][0];
    assert!(brightness > warmth * 1.15, "amplitudes={amplitude:?}");
    println!(
        "COLOR_HF_CHANGE_DB: {:.2}",
        20.0 * (brightness / warmth).log10()
    );
}
