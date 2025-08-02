use bevy::prelude::*;
pub struct PawnPlugin;

impl Plugin for PawnPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_pawns);
        // app.add_systems(Update, rotate_on_drag);
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
        ))
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

/// An observer to rotate an entity when it is dragged
fn rotate_on_drag(drag: Trigger<Pointer<Drag>>, mut transforms: Query<&mut Transform>) {
    let mut transform = transforms.get_mut(drag.target()).unwrap();
    transform.translation += Vec3::new(drag.delta.x, -drag.delta.y, 0.0);
    // transform.rotate_y(drag.delta.x * 0.02)
    // transform.rotate_x(drag.delta.y * 0.02);
}
