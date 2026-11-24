# A save is one zip per game

A save holds `mimetype`, `manifest.json` (the whole document, plus an entry per image) and
`assets/<id>.<ext>`, with extension `.ksg` and media type `application/vnd.kriegsspiel+zip`.
Images are stored uncompressed and the manifest deflated. All of it is written and read in
Rust; the client never unpacks a save.

It is the same shape as `.docx`, `.ora` and `.usdz`. The document is a rounding error next to
the images (measured: 1.1 KB of overhead on a 15.7 MB save), so it can stay readable JSON,
which keeps the format tolerant of fields still to be added (`props`, scripts, messages).
`manifest.json` holds the document, not just an index.

## Considered options

- **A hand-rolled binary container.** It would save nothing, because the images are already
  compressed, and every added field would become a codec change. A zip also gives random
  access and `unzip` for debugging.
