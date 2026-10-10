# EmberFat

[![Windows build](https://github.com/0xPRISMM/EmberFat/actions/workflows/windows-release.yml/badge.svg)](https://github.com/0xPRISMM/EmberFat/actions/workflows/windows-release.yml)

EmberFat is a stereo saturation effect built with Rust and [NIH-plug](https://github.com/robbert-vdh/nih-plug). It provides four focused controls for shaping drive, tone, blend, and output level.

## Features

- FAT drive and saturation
- COLOR harmonic character
- MIX dry/wet blend
- OUTPUT level control
- Bypass and double-click reset
- VST3 plugin with stereo I/O

## Download for Windows

1. Open the [Windows build workflow](https://github.com/0xPRISMM/EmberFat/actions/workflows/windows-release.yml).
2. Open the latest successful run and download its `EmberFat-Windows-*` artifact.
3. Extract the ZIP and run `EmberFat-Windows-Setup.exe`.
4. Choose the installation folder, then rescan VST3 plugins in your DAW.

Workflow artifacts are temporary downloads and expire after 30 days. Versioned public releases are not published yet.

## Build from source

Install the stable Rust toolchain and a native C/C++ build toolchain for your platform, then run:

```sh
cargo test --workspace
cargo run --package xtask -- bundle emberfat --release
```

The VST3 bundle is generated under `target/bundled/`. To create the Windows installer locally, install Inno Setup 6 and compile `installer/windows/EmberFat.iss` after staging the bundle under `dist/EmberFat.vst3`.

## Repository layout

```text
.github/workflows/   CI build and release artifact workflow
docs/                Release roadmap and project notes
installer/windows/   Inno Setup installer definition
plugin/              EmberFat plugin source and manifest
xtask/               NIH-plug bundling task
```

Build outputs and installers are generated in `target/` and `dist/`; they are intentionally not stored in the source repository.

## Licensing

The plugin crate declares `GPL-3.0-or-later` in `plugin/Cargo.toml`. Review NIH-plug and dependency licensing before redistributing builds.
