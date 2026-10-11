# EmberFat 0.2.0 DSP review

The reported stutter occurs while turning controls. The previous processor read each parameter once per host block and changed the transfer function immediately. This makes block boundaries audible when drive, blend or gain changes. Bypass also switched instantly. The revised processor interpolates controls per sample and crossfades bypass, while retaining stereo coherence.

## Findings and changes

| Previous behavior | Revised behavior |
| --- | --- |
| FAT/COLOR/MIX/OUTPUT changed abruptly | 20 ms ramps, independent of host buffer size; sample-accurate automation event delivery |
| Bypass jumped between dry and wet | 5 ms crossfade, with the processing state kept warm |
| Nonlinear processing at the host sample rate | Fixed 4x oversampling using two cascaded, symmetric 65-tap half-band FIR stages |
| Dry/wet paths had no oversampling alignment | Both use a fixed 48-sample delay, reported to the host |
| The claimed even-harmonic term was odd: `x*x*sign(x)` | Bias in an original compensated tanh curve produces even harmonics; the zero-input bias is subtracted |
| Drive normalization strongly boosted quiet signals | Progressive density and gentler gain compensation |
| FAT=0 still applied saturation | FAT=0 at unity OUTPUT is exactly clean |
| The final clamp clipped audio even at MIX=0 | Clean/blended audio retains floating-point headroom; only extreme overflow is guarded |
| No DC handling for asymmetric distortion | 10 Hz DC rejection on the added distortion; clean bass is preserved |

COLOR also smooths the added distortion through a sample-rate-aware tone filter. The source implements the ramps, FIR coefficient design, fixed-size history buffers and saturation curve specifically for EmberFat. The audio path allocates no memory, takes no locks and performs no file or network I/O.

## GitHub comparisons

- [NIH-plug Soft Vacuum](https://github.com/robbert-vdh/nih-plug/tree/master/plugins/soft_vacuum) adds parameter smoothing and configurable linear-phase oversampling to a saturation processor. Its source also reports oversampling latency to the host. These support smoothing, anti-aliasing and delay compensation as appropriate remedies. EmberFat does not use its Hard Vacuum algorithm, Lanczos oversampler or source code.
- [Airwindows PurestDrive](https://github.com/airwindows/airwindows/blob/master/plugins/WinVST/PurestDrive/PurestDriveProc.cpp) uses sine shaping and signal-dependent blending involving previous samples. This illustrates a different, dynamics-sensitive approach to density. EmberFat retains its own progressive biased tanh design; no PurestDrive code or noise/dither generator was imported.

These are engineering comparisons, not claims that one plugin sounds better than another. The new curve and gain compensation intentionally change EmberFat's sound; existing presets keep their parameter identities but may need level adjustment.

## Validation

17 workspace tests pass, including the existing editor interaction tests. Audio tests cover automation at 44.1/48/96 kHz, stereo equality, channel isolation, reset, exact clean-path latency, headroom, finite output after invalid samples, DC rejection, genuine even harmonics and an alias-reduction regression.

The actual packaged Windows VST3 was also loaded by a headless host using its component/controller/audio interfaces. Timed VST3 parameter queues produced identical samples with 7, 64, 128 and 512-frame host buffers. The packaged plugin preserved a +3.0/-2.0 stereo impulse exactly at MIX=0 and reported 48 samples of latency at 44.1, 48 and 96 kHz.

| Measurement | Result and scope |
| --- | --- |
| Folded harmonics, 9 kHz sine, 48 kHz, FAT=80%, COLOR=40% | Selected alias-bin amplitude: -65.9 dBFS at 4x versus -26.3 dBFS for the same curve at 1x; **39.6 dB reduction**. Fundamental amplitudes were approximately 0.385 in both versions. This is one stress case, not a broadband guarantee. |
| FAT/COLOR/gain step on a 0.2 constant carrier | Maximum adjacent change 0.000224 with smoothing versus 0.13018 without it, about **55 dB smaller**. This measures control discontinuity, not perceived loudness. |
| Packaged VST3 parameter step | Maximum adjacent change 0.000244, below the 0.002 test threshold |
| Optimized stereo DSP, local CPU | 10 seconds of 48 kHz audio processed in 0.17-0.19 seconds; approximately 1.7-1.9% of one core in this offline run |
| Python host with 128-frame blocks | Mean call time 0.224-0.325 ms, 99th percentile 0.622-1.851 ms across two runs; audio deadline 2.667 ms. Includes Python/dispatch overhead; not a DAW performance guarantee. |

The before/after WAV examples use an original synthesized musical phrase, the original 0.1.7 transfer equation and the packaged 0.2.0 processor. They are RMS matched and given common peak headroom. They illustrate the sound change without letting a large volume difference decide the comparison.

The tests establish numerical behavior and host integration. They cannot confirm the result in the user's DAW or guarantee a subjective professional sound. The remaining listening check is to turn each control in that DAW after installing 0.2.0 and assess both noise and character.

## Interface update

The owner's latest background is embedded unchanged. The OUTPUT knob and its label/value group move 12 pixels right in the 1000x540 layout. OUTPUT uses a 17 px title and 14.5 px readout; the existing Barlow Condensed style is retained. The README image is rendered from the editor's actual paint meshes and textures.
