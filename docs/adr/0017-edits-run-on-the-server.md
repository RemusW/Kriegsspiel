# Edits run on the server

Umpire clients send commands (move, rotate, undo, record turn and so on) and display the
boards the server sends back; there is no client-side `Doc`. With several editors, client-side
copies would drift apart, while commands are small, can be checked, and do not overwrite each
other. Remote umpires need their pending edits shown locally to feel instant, which
`PendingEdits` provides.
