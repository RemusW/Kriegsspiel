//! Standalone game server: serves the demo game on localhost.

use std::net::{Ipv4Addr, SocketAddr};

use kriegsspiel_server::{DEFAULT_PORT, Game};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let addr = SocketAddr::from((Ipv4Addr::LOCALHOST, DEFAULT_PORT));
    println!("game server listening on ws://{addr}/ws");
    kriegsspiel_server::serve(addr, Game::demo()).await
}
