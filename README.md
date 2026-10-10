<img width="516" height="298" alt="ember" src="https://github.com/user-attachments/assets/8618a86e-f333-4116-8256-89814e51a9f2" />
# EmberFat — Rust VST3 saturation plugin

EmberFat is an original, Sausage Fattener-inspired saturation effect. It does not use or copy proprietary Sausage Fattener code, branding, or presets.

## Features

- VST3 plugin export through NIH-plug
- Custom dark GUI with large red rotary controls
- FAT: input drive / saturation amount
- COLOR: harmonic tone character
- MIX: dry/wet blend
- OUTPUT: output level in dB
- BYPASS
- Stereo audio I/O
- Smooth tanh saturation with a subtle color stage

## Requirements

- Rust stable toolchain (install from https://rustup.rs/)
- A C/C++ build toolchain supported by your operating system
- Windows, macOS, or Linux
- FL Studio with VST3 support

## Build a VST3 bundle

From this directory, run:

```bash
cargo xtask bundle emberfat --release
```

The NIH-plug bundler should place the generated VST3 bundle under `target/bundled/`. Copy `EmberFat.vst3` into your system's VST3 plugin directory if it was not installed there automatically, then rescan plugins in FL Studio.

To compile-check the Rust crate without bundling:

```bash
cargo check -p emberfat
```

## Windows VST3 location

A common per-user location is:

```text
%LOCALAPPDATA%\\Programs\\Common\\VST3
```

The system-wide VST3 directory is commonly:

```text
C:\\Program Files\\Common Files\\VST3
```

## Controls

- **FAT**: raises the drive into a smooth saturation curve.
- **COLOR**: adds a subtle tone/harmonic adjustment before saturation.
- **MIX**: blends dry and processed signal.
- **OUTPUT**: adjusts post-effect level.
- **BYPASS**: passes audio unchanged.

Double-click a knob to reset it to its default value. Drag vertically to change its value.

## Notes

- This source project has not been compiled or tested in FL Studio in this environment.
- A VST3 bundle is platform-specific, so build it on the operating system you intend to use.
- NIH-plug's VST3 bindings have GPL-3.0 licensing implications. Review the framework license and its current repository guidance before distributing binaries or source.
