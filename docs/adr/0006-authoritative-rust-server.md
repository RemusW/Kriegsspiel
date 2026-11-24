# An authoritative server in Rust

The game server holds the true state and decides every change. Hidden information has to be
filtered on the server, so a client never holds more than its side may see.
