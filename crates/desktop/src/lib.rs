//! Umpire desktop application: the web client in a window, with the game server running
//! inside the same process.

use std::net::{Ipv4Addr, SocketAddr};

use kriegsspiel_server::{DEFAULT_PORT, Game};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default();
    #[cfg(all(debug_assertions, target_os = "macos"))]
    let builder = reload_menu::install(builder);

    builder
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

#[cfg(all(debug_assertions, target_os = "macos"))]
mod reload_menu {
    use tauri::menu::{Menu, MenuItem, Submenu};
    use tauri::{Builder, Manager, Wry};

    const RELOAD: &str = "reload";

    pub fn install(builder: Builder<Wry>) -> Builder<Wry> {
        builder
            .menu(|app| {
                let menu = Menu::default(app)?;
                let reload = MenuItem::with_id(app, RELOAD, "Reload", true, Some("CmdOrCtrl+R"))?;
                menu.append(&Submenu::with_items(app, "Develop", true, &[&reload])?)?;
                Ok(menu)
            })
            .on_menu_event(|app, event| {
                if event.id() == RELOAD
                    && let Some(window) = app.get_webview_window("main")
                    && let Err(err) = window.reload()
                {
                    eprintln!("reload failed: {err}");
                }
            })
    }
}
