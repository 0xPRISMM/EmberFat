# EmberFat release roadmap

Target branch: `master` (work is staged on `feature/windows-release-pipeline` for review).

## Product direction

EmberFat is a focused saturation effect, not a synth or channel-strip dashboard. Keep the four primary controls—FAT, COLOR, MIX, OUTPUT—prominent. Use a distinctive hybrid hardware/studio visual language: tactile custom knobs, restrained metal detailing, charcoal background, and the existing warm red accent. Avoid large analyzers, waveform displays, modulation panels, and unnecessary secondary controls.

## Non-negotiable audio-engine guardrails

- Preserve the current `Plugin::process` implementation and parameter IDs while doing the first GUI/release pass.
- Do not add DSP modes, oversampling, filters, or automatic gain compensation in the initial GUI commit.
- Test bypass, dry/wet, parameter automation, saved-state recall, stereo routing, and output-level behavior in the target DAW before merging.
- Any later DSP change must be isolated in its own commit and A/B-tested against the current release.

## Delivery phases

1. **Baseline and release automation** — record the known-good `master` state, build on Windows CI, retain the VST3 and installer as artifacts.
2. **Focused GUI refresh** — improve knob rendering, typography, spacing, hover/drag feedback, and value display without changing audio processing.
3. **Usability and presets** — confirm reset behavior, parameter automation/state persistence, and a small set of factory presets only after host testing.
4. **Windows installer hardening** — validate install/uninstall, VST3 folder layout, clean-machine install, and common DAW rescans.
5. **Public release** — add versioned releases, checksums, release notes, signing if certificates are available, and verify all framework/licensing obligations.

## Current Windows pipeline

The workflow at `.github/workflows/windows-release.yml` runs on manual dispatch and version tags. It builds the VST3 bundle with the repository's xtask, stages it, compiles the Inno Setup installer, and uploads both as CI artifacts. It does not publish a public release automatically.

## Known limitations / follow-ups

- Confirm the exact directory layout produced by `cargo xtask bundle emberfat --release` on Windows; the workflow intentionally fails if it cannot find `EmberFat.vst3`.
- Keep installer version synchronized with `plugin/Cargo.toml` before a public release.
- Code signing and Windows SmartScreen reputation require external certificates/account setup and are not enabled here.
- Run the workflow and install the artifact on a clean Windows machine before treating it as release-ready.
- Review NIH-plug/VST3 licensing and attribution before distributing binaries.
