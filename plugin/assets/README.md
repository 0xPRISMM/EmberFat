# EmberFat interface artwork



The editor embeds the project owner's supplied PNG artwork as unpremultiplied

RGBA8 textures without resampling. PNG sources are retained alongside

raw byte textures. Assets are embedded in the plugin; no runtime downloads.



- amber-panel: 1709 × 920 faceplate, with the EMBERFAT wordmark already included.

- control-sheet: 2048 × 768 transparent sheet with three sizes of metal caps,

  a small output cap, and static range markings. UV sprite rectangles and

  rotation pivots are declared in editor.rs. The supplied ivory pointers rotate

  with the caps; range markings remain fixed.

- switch-sheet: 2048 × 768 transparent active (orange) and bypass (gray) plates.

- tiger-sheet: 2172 × 724 transparent sheet with six progressively larger tigers.

  The sheet is split into two texture pages at x=1008 to fit 2048-pixel GPUs.
  All stages share the same display scale and baseline, selected by FAT.

- fonts/BarlowCondensed-SemiBold.ttf: unmodified OFL 1.1 font; see fonts/OFL.txt

  and ../../THIRD_PARTY_NOTICES.md.



Earlier graphite artwork remains in the source tree but is no longer embedded.


Control captions use Barlow Condensed SemiBold again. Knob title sizes are
26.6 px (FAT/COLOR), 19 px (MIX), and 17 px (OUTPUT). Readouts are
16.5 px except OUTPUT at 14.5 px. OUTPUT and its label group move 12 px right. The Permanent Marker font and
license are retained as unused source assets from the earlier style variant.
