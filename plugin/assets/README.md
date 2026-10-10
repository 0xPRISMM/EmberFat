# EmberFat interface artwork

The plugin embeds these tightly packed, unpremultiplied RGBA8 textures so it can render without an image-decoding dependency. Each texture's pixel dimensions are declared beside its `include_bytes!` call in `plugin/src/lib.rs`.

- `metal-background-900x486.rgba` is the supplied metal panel background.
- `fat-stage-1.rgba` through `fat-stage-6.rgba` are the six transparent mascot poses selected by the FAT control.
- `red-metal-knob-256.rgba` is the supplied dial face; `knob-range-256.rgba` is its separate range outline.
- `output-knob-72.rgba` is a darker compact variant for OUTPUT.
