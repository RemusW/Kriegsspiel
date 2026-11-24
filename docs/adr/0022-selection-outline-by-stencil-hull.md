# The selection outline is a stencil-masked hull

The selected mesh is drawn again as a hull, its vertices pushed outward on screen along
normals averaged over shared positions, without a depth test and only where the blocks have
not written stencil 1. Chosen by the owner over a post-process mask, because it needs no
render pipeline.

## Considered options

- **A depth-tested hull** lost the line where a block meets the table.

## Consequences

The outline shows through anything that does not call `maskOutlines`, so future terrain and
models must call it to hide outlines behind them.
