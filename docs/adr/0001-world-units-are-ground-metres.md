# World units are metres on the ground

One world unit is one metre of real ground. Each map carries its own ground scale, in metres
per image pixel, and its extent on the table is its pixel size times that scale. A unit's
footprint is in metres too, from its kind and formation. Kriegsspiel is played by measuring
march distances and ranges on the map, so maps of different scales and the pieces on them must
agree about how much ground they cover.

The ground scale is set by calibration: the umpire marks both ends of the map's own scale bar
and says what distance it stands for. A new map starts at 1 m per pixel until then.

## Considered options

- **Abstract world units, every map laid at a fixed width** (what came before). Two maps of
  different scales then disagree about how much ground a unit covers, and nothing measured on
  the table means anything.
- **Ground scale from the image's DPI header and a scale ratio.** The DPI cannot be trusted:
  resizing an image leaves it stale or drops it, screenshots carry a nominal 72 or 96, and a
  photo of a paper map has none that means anything. A scale bar printed on the map is resized
  and photographed with it, so it always matches the pixels.

## Consequences

Every distance tuned in arbitrary units (camera speeds, the blank table's size, the pawn
springs) has to be retuned once. f32 keeps millimetre precision out to about 10 km.
