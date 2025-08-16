use crate::actions::action::{ActionManager, Movable, MoveAction};
use bevy::{prelude::*, render::view::RenderLayers};
pub struct PawnPlugin;

impl Plugin for PawnPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_pawns);
    }
}

#[derive(Component)]
struct Pawn;

pub fn spawn_pawns(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mesh = meshes.add(Rectangle::new(20.0, 10.0));
    let material = materials.add(Color::srgb(1.0, 0.0, 0.0));

    commands
        .spawn((
            Transform::from_xyz(0.0, 0.0, 0.0),
            GlobalTransform::default(),
            Mesh2d(mesh),
            MeshMaterial2d(material),
            Pawn,
            Movable,
            RenderLayers::layer(0),
        ))
        .observe(handle_move_drag)
        .observe(rotate_on_drag);
}

// fn draw_pawns(mut query: Query<&mut Transform, With<Pawn>>) {
// for mut name in &mut query {
//     if name.0 == "Elaina Proctor" {
//         name.0 = "Elaina Hume".to_string();
//         break; // We don't need to change any other nam's.
//     }
// }
// }

fn handle_move_drag(
    drag: Trigger<Pointer<DragEnd>>,
    mut action_manager: ResMut<ActionManager>,
    transforms: Query<&Transform>,
    mut commands: Commands,
) {
    if let Ok(transform) = transforms.get(drag.target()) {
        let to_transform = *transform;
        let mut from_transform = to_transform.clone();
        from_transform.translation -= Vec3::new(drag.distance.x, -drag.distance.y, 0.0);
        let move_action = Box::new(MoveAction {
            entity: drag.target(),
            from: from_transform,
            to: to_transform,
        });
        action_manager.execute(move_action, &mut commands);
    }
}

fn rotate_on_drag(drag: Trigger<Pointer<Drag>>, mut transforms: Query<&mut Transform>) {
    let mut transform = transforms.get_mut(drag.target()).unwrap();
    transform.translation += Vec3::new(drag.delta.x, -drag.delta.y, 0.0);
}