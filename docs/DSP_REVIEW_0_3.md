# EmberFat 0.3.0: stronger FAT processing

The 0.2.0 processor was deliberately gentle: drive compensation cancelled much of its boost, and it had no envelope-based compressor. This version gives FAT a substantially wider range while retaining smooth controls and anti-aliasing.

## Processing

- FAT controls an original stereo-linked feed-forward compressor: a 6 dB knee, threshold progressing from -10 to -28 dB, and ratio from 1:1 to 12:1. The detector uses 2 ms attack and 80 ms release.
- FAT also adds up to 10 dB of makeup gain and a progressively stronger nonlinear drive. It blends soft saturation with a rounded, continuous-slope clipper. It no longer normalizes away the intended loudness and density.
- COLOR emphasizes high frequencies before the nonlinear stage and opens the added distortion's high-frequency response. It also affects asymmetry and clipping character.
- The nonlinear stage now runs at 8x sample rate. Three cascaded half-band FIR stages provide anti-alias filtering. Distortion-generated DC is rejected without filtering the clean bass path.
- A stereo-linked peak controller uses 24 samples of lookahead, 0.1 ms attack and 60 ms release. Its action blends in with FAT; at maximum FAT, the internal wet signal is bounded to +/-1 before OUTPUT.
- OUTPUT now ranges from -18 to **+24 dB**. It is deliberately after internal peak control and before MIX, so positive OUTPUT can raise samples above 0 dBFS. MIX and bypass retain clean-signal headroom.
- All paths have a fixed **80-sample latency**, reported to the host. Knobs still ramp over 20 ms and bypass crossfades over 5 ms. The audio path uses fixed arrays and does not allocate or lock.
- The VST3 subcategory is **Fx only**, making EmberFat a general audio effect rather than a distortion-specific effect. A host may require a rescan to update its cached category.

The [official Sausage Fattener description](https://dadalife.com/sausage-fattener-plugin/) describes moderate settings as compression and higher settings as dirty distortion. Its published description does not disclose the complete algorithm. This revision targets that broader behavioral range with original code; it is not an emulation or an exact sound match.

## Measurements against the actual 0.2.0 VST3

Both packaged plugins processed the same 100 Hz sine at peak amplitude 0.03, 48 kHz, COLOR=28%, MIX=100%, OUTPUT=0 dB. Values below measure the last second after settling. No output level matching was applied in this table.

| FAT | RMS 0.2.0 | RMS 0.3.0 | Level increase vs 0.2.0 | THD 0.2.0 | THD 0.3.0 |
| --- | ---: | ---: | ---: | ---: | ---: |
| 0% | 0.02121 | 0.02121 | 0.0 dB | effectively zero | effectively zero |
| 25% | 0.02433 | 0.05237 | +6.7 dB | 0.047% | 0.117% |
| 50% | 0.02821 | 0.24620 | +18.8 dB | 0.227% | 1.668% |
| 100% | 0.03195 | 0.85012 | +28.5 dB | 0.944% | 27.911% |

At maximum FAT, this tone's crest factor drops from 3.04 to 1.41 dB: the change adds density and harmonics, as well as level. These are deliberately input-dependent measurements, not a gain guarantee on every source.

On an original synthesized percussion/bass phrase at FAT=70%, COLOR=35%, OUTPUT=-1 dB, the new processor adds 20.3 dB of RMS level relative to 0.2.0 and reduces crest factor from 14.08 to 7.29 dB. The supplied comparison WAVs are RMS matched with common peak headroom to let listeners compare character without a volume advantage.

## Validation

22 workspace tests pass. They cover clean-path transparency/headroom, FIR latency, stereo/channel isolation, reset, silence and finite input recovery, genuine even harmonics and DC rejection, linked compression, peak control, increased bass density, COLOR brightness, the full +24 dB gain ramp, and automation independent of block size.

- A same-curve 9 kHz anti-aliasing stress test, holding compressor gain constant in both versions, measures **40.95 dB less selected folded-harmonic energy** with the 8x filters than at 1x. This isolates the filters; it is not a broadband or whole-plugin guarantee.
- The COLOR test increases the 6 kHz-to-600 Hz fundamental ratio by 3.53 dB, at FAT=30% and a low-level input.
- The actual packaged VST3 reports -18/+24 dB OUTPUT and Fx-only metadata. It reproduces identical timed automation at 7, 64, 128 and 512-frame buffers, and reports 80 samples of latency at 44.1, 48 and 96 kHz.
- The +24 dB simultaneous control-step host test has a maximum relative adjacent change of 0.22% of its measured peak. A separate test verifies the exact 20 ms linear gain-ramp slope at the full boost range.
- Ten seconds of optimized stereo DSP took 0.515 seconds on the local CPU, about 5.2% of one core in this offline run. A Python VST3 host with 128-frame buffers measured mean processing-call time 0.304 ms and 99th percentile 0.776 ms, below its 2.667 ms audio deadline. Actual DAW performance depends on the system and session.

Parameter IDs and the VST3 class ID remain stable. The sound, latency and OUTPUT range change intentionally; existing OUTPUT automation may need adjustment to the extended range. The interface artwork and layout remain the same. Listening against Sausage Fattener itself was not performed; this report compares EmberFat's own released processors.
