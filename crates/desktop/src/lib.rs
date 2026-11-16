//! Umpire desktop application: the web client in a window, with the game server running
//! inside the same process.

use std::net::{Ipv4Addr, SocketAddr};

use kriegsspiel_server::{DEFAULT_PORT, Game};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|_app| {
            let addr = SocketAddr::from((Ipv4Addr::LOCALHOST, DEFAULT_PORT));
            tauri::async_runtime::spawn(async move {
                if let Err(err) = kriegsspiel_server::serve(addr, Game::demo()).await {
                    eprintln!("game server stopped: {err}");
                }
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
