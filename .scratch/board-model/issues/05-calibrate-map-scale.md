# Calibrate a map against its scale bar

Status: ready-for-agent
Blocked by: 04, 07

## What

- Server: `setMapScale { map, metresPerPixel }`, an ordinary undoable edit. Refuses values
  that are not finite and positive. Recomputes `size` from the image's pixel size (from the
  asset store, since the board does not keep pixels), keeping the map's centre.
- Client: a calibrate action in map editing mode. The umpire clicks both ends of the map's
  scale bar, then types the distance it stands for with a unit (mile, yard, pace, verst,
  km, m). The client converts to metres, works out metres per pixel from the two points and
  the map's current scale, and sends the command. Units of length live only in the client;
  the board stores metres.

## Why

ADR 0001: the scale bar is printed on the map and resized with it, so it always matches the
pixels, unlike an image's DPI.

## Acceptance

- Worked example as a test: a scale bar 1 mile long spanning 540 px gives 2.98 m/px.
- Undo puts the old scale and size back.
