# Release roadmap

## Current

- Windows CI runs workspace tests, builds the VST3 bundle, compiles the installer, and uploads both as a 30-day workflow artifact.
- The GUI has four primary controls, drag and Shift-drag adjustment, double-click reset, bypass, and host parameter gestures.

## Before a public release

- Validate the plugin in a supported DAW: bypass, dry/wet, automation, saved-state recall, stereo routing, and output level.
- Verify installer install/uninstall behavior and rescan the plugin in common DAWs.
- Synchronize the installer version with `plugin/Cargo.toml`.
- Review NIH-plug and dependency licensing and add any required attributions.
- Add versioned release notes and checksums. Consider code signing if a certificate is available.

See the [Windows build workflow](https://github.com/0xPRISMM/EmberFat/actions/workflows/windows-release.yml) for current build artifacts.
