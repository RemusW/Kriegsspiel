# Shared types are generated from Rust by ts-rs

The client's board and message types are generated from the Rust structs, so the TypeScript
compiler flags every client line a Rust change breaks. `.cargo/config.toml` sets the output
folder and exports `u64` as `number`, because serde_json sends `u64` as a JSON number and
ts-rs's default `bigint` would be wrong.
