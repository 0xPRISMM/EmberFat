# EmberFat interface artwork

The runtime artwork is stored as plain RGBA pixel data so the plugin can render it without adding an image-decoding dependency. Each `*.rgba` file contains tightly packed, unpremultiplied RGBA8 pixels; the dimensions are specified at each `include_bytes!` call in `plugin/src/lib.rs`.

The copper knob and six mascot poses were prepared from images supplied for the EmberFat GUI. FAT selects the mascot pose, and the copper knob pointer rotates with each parameter.
