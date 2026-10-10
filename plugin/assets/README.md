# EmberFat interface artwork

The editor embeds unpremultiplied RGBA8 textures; dimensions are declared beside
each include_bytes! call in plugin/src/editor.rs. It has no runtime asset downloads.

- metal-background-900x486.rgba: supplied metal panel, sampled without its transparent outer margins.
- fat-stage-1.rgba through fat-stage-6.rgba: the six supplied mascot stages.
- silver-knob-384.rgba: silver, front-facing dial base derived from the supplied knob.
  The image stays fixed; the ivory indicator is drawn and rotated by the editor,
  so the circular housing and lighting cannot wobble when adjusting a value.
- knob-range-256.rgba: supplied range outline, rendered in neutral gray.
- fonts/BarlowCondensed-SemiBold.ttf: unmodified OFL 1.1 font; see fonts/OFL.txt
  and ../../THIRD_PARTY_NOTICES.md.

The compact OUTPUT control is drawn as a circular gray gain cap in the editor.

## Silver knob generation

Built-in imagegen, precise-object-edit, from the project owner's knob.
Final prompt: strict orthographic top view; geometrically circular, concentric
silver brushed disc and dark gray grip with equal radial width; remove the ivory
indicator for separate rendering; preserve mechanical detail; fixed neutral light;
transparent background; no perspective, bottom extrusion, labels or colored glow.
The generated silhouette is fitted to a square texture for equal horizontal and
vertical diameter. This project artwork is separate from the third-party font.
