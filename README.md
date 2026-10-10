# EmberFat

[![Windows build](https://github.com/0xPRISMM/EmberFat/actions/workflows/windows-release.yml/badge.svg)](https://github.com/0xPRISMM/EmberFat/actions/workflows/windows-release.yml)

EmberFat is a stereo saturation effect built with Rust and [NIH-plug](https://github.com/robbert-vdh/nih-plug). It provides four focused controls for shaping drive, tone, blend, and output level.

![EmberFat interface](docs/emberfat-ui.png)

## Controls

- FAT drive and saturation
- COLOR harmonic character
- MIX dry/wet blend
- OUTPUT processed-signal level, before the dry/wet blend
- ACTIVE/BYPASSED comparison and double-click reset
- Shift-drag for fine control, click a value to type, arrow keys and mouse wheel adjustment
- VST3 plugin with stereo I/O

## Download for Windows

1. Open the [Windows build workflow](https://github.com/0xPRISMM/EmberFat/actions/workflows/windows-release.yml).
2. Sign in to GitHub, open the latest successful run, and download its `EmberFat-Windows-*` artifact.
3. Extract the ZIP and run `EmberFat-Windows-Setup.exe`.
4. Close your DAW, choose the VST3 installation folder, then reopen the DAW and rescan plugins.

Each download includes `BUILD.txt` with its version/source revision and `SHA256SUMS.txt` for the installer. The installer version comes from the plugin manifest.

Workflow artifacts are temporary downloads and expire after 30 days. Versioned public releases are not published yet.

## Build from source

Install the stable Rust toolchain and a native C/C++ build toolchain for your platform, then run:

```sh
cargo test --workspace
cargo run --package xtask -- bundle emberfat --release
```

The VST3 bundle is generated under `target/bundled/`. For the complete Windows package, install Inno Setup 6 and run:

```powershell
./scripts/package-windows.ps1
```

Use `-InnoCompiler 'C:/path/to/ISCC.exe'` for a custom installation. This builds the VST3, stages a fresh bundle with UI asset licenses, and creates the installer and checksum in `dist/`.

## Repository layout

```text
.github/workflows/   CI build and release artifact workflow
docs/                Release roadmap and project notes
installer/windows/   Inno Setup installer definition
plugin/              EmberFat plugin source and manifest
scripts/             Repeatable Windows packaging
xtask/               NIH-plug bundling task
```

Build outputs and installers are generated in `target/` and `dist/`; they are intentionally not stored in the source repository.

## Licensing

Barlow Condensed is embedded under the SIL Open Font License 1.1; see [UI asset notices](THIRD_PARTY_NOTICES.md). The plugin crate declares `GPL-3.0-or-later` in `plugin/Cargo.toml`. Review NIH-plug and dependency licensing before redistributing builds.
