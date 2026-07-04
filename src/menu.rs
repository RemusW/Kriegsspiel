use egui_macroquad::egui::{self, menu};
use macroquad::prelude::*;
use macroquad::ui::{root_ui, widgets};

use crate::asset::AssetStore;
use crate::command::CommandManager;
use crate::editor::{EditorState, ToolMode};
use crate::sprite::Sprite;
use crate::{AppContext, PIXELS_PER_UNIT, spawn_pawn};

// #[derive(Default)]
enum AppState {
    Menu(Menu),
    Editor(Editor),
}

impl Default for AppState {
    fn default() -> Self {
        AppState::Menu(Menu::default())
    }
}

pub struct SceneManager {
    pub app_state: AppState,
}

impl SceneManager {
    pub fn new() -> Self {
        Self {
            app_state: AppState::default(),
        }
    }

    pub fn run(&mut self, ctx: &mut AppContext) {
        let next_scene = match &mut self.app_state {
            AppState::Menu(menu) => menu.update(),
            AppState::Editor(editor) => editor.update(ctx),
        };

        if let Some(scene) = next_scene {
            self.transition(scene);
        }
    }

    fn transition(&mut self, new_state: AppState) {
        match &new_state {
            AppState::Menu(menu) => menu.enter(),
            AppState::Editor(editor) => editor.enter(),
        }
        self.app_state = new_state;
    }
}

pub enum MenuAction {
    Start,
    Load,
}

#[derive(Default)]
struct Menu {}

impl Menu {
    pub fn update(&self) -> Option<AppState> {
        clear_background(LIGHTGRAY);

        let center = vec2(screen_width() / 2.0, screen_height() / 2.0);
        let button_size = vec2(160.0, 40.0);

        if widgets::Button::new("Start")
            .position(center - vec2(button_size.x / 2.0, button_size.y + 10.0))
            .size(button_size)
            .ui(&mut root_ui())
        {
            menu_start();
        }

        if widgets::Button::new("Load")
            .position(center - vec2(button_size.x / 2.0, 0.0))
            .size(button_size)
            .ui(&mut root_ui())
        {
            menu_load();
        }
        None
    }

    pub fn enter(&self) {}
}

fn menu_start() {}

fn menu_load() {}

#[derive(Default)]
struct Editor {
    editor_state: EditorState,
}

impl Editor {
    pub fn update(&mut self, ctx: &mut AppContext) -> Option<AppState> {
        clear_background(LIGHTGRAY);

        let mut world = &mut ctx.world;
        let mut command_manager = &mut ctx.command_manger;

        let cavalry = ctx.asset_store.get_handle_from("cavalry");
        let cavalry = Sprite::new(cavalry, &ctx.asset_store);

        let mut save_world = false;
        egui_macroquad::ui(|ectx| {
            egui::Window::new("Actions")
                .resizable(false)
                .show(ectx, |ui| {
                    ui.label("Mouse Mode");
                    ui.separator();
                    for mode in [ToolMode::Spawn, ToolMode::Move] {
                        ui.selectable_value(&mut self.editor_state.tool_mode, mode, mode.label());
                    }
                    ui.separator();
                    if ui.button("Save World").clicked() {
                        save_world = true;
                    }
                });
        });

        if save_world {
            // JSON
            // match serde_json::to_string_pretty(&world) {
            //     Ok(data) => match std::fs::write("assets/world.json", data) {
            //         Ok(()) => println!("Saved world to assets/world.json"),
            //         Err(e) => eprintln!("Failed to write world: {e}"),
            //     },
            //     Err(e) => eprintln!("Failed to serialize world: {e}"),
            // }

            // RON — comment out the JSON block above and uncomment this to switch
            match ron::ser::to_string_pretty(&world, ron::ser::PrettyConfig::default()) {
                Ok(data) => match std::fs::write("assets/world.ron", data) {
                    Ok(()) => println!("Saved world to assets/world.ron"),
                    Err(e) => eprintln!("Failed to write world: {e}"),
                },
                Err(e) => eprintln!("Failed to serialize world: {e}"),
            }
        }

        if let Some(ref map) = world.map {
            map.draw_default();
        }

        self.editor_state.update(&mut world, &mut command_manager);
        if self.editor_state.tool_mode == ToolMode::Spawn {
            spawn_pawn(&mut world, &cavalry);
        }

        draw_circle(0.0, 0.0, 50.0, WHITE);
        draw_line(40.0, 40.0, 100.0, 200.0, 15.0, BLUE);
        draw_line(-0.4, 0.4, -0.8, 0.9, 10.0, BLUE);

        // Draw highlight box around pawn if selected
        let pawns = self.editor_state.selection.get_selected_pawns(&world);
        for pawn in pawns.iter() {
            pawn.draw_highlight();
        }

        for ele in world.pawn_manager.pawn_map.iter() {
            ele.1.draw();
        }

        world.camera.update();

        set_default_camera();

        draw_text(
            &format!(
                "Window: {}x{}  |  World units visible: {:.1} x {:.1}",
                screen_width() as i32,
                screen_height() as i32,
                screen_width() / PIXELS_PER_UNIT,
                screen_height() / PIXELS_PER_UNIT,
            ),
            10.0,
            20.0,
            20.0,
            WHITE,
        );
        draw_text(&format!("{:?}", mouse_position()), 10.0, 40.0, 20.0, WHITE);

        // Debug information
        egui_macroquad::draw();

        None
    }

    fn enter(&self) {}

    fn debug_enter(&self, ctx: &mut AppContext) {
        let asset_store = &mut ctx.asset_store;
        let cavalry = asset_store.load_texture("cavalry", "assets/cavalry.png");
        let farley = asset_store.load_texture("farley", "assets/farley.png");

        // cavalry.set_scale(5.0, 5.0);

        let mut farley = Sprite::new(farley, &asset_store);
        farley.set_size(1000.0, -1000.0);

        ctx.world.map = Some(farley);
    }
}
