use crate::actions::action::{ActionManager, Movable, MoveAction};
use bevy::{
    ecs::system::entity_command::observe, input::mouse::MouseButtonInput,
    log::tracing_subscriber::filter::targets, prelude::*, render::view::RenderLayers,
};
pub struct PawnPlugin;

impl Plugin for PawnPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_pawns);
        app.add_systems(Update, drag_update);
    }
}

#[derive(Component)]
struct Pawn;

pub fn spawn_pawns(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mesh = meshes.add(Rectangle::new(100.0, 50.0));
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
        .observe(record_drag)
        .observe(selected_update)
        .observe(unselected_update);
}

fn record_drag(
    drag: Trigger<Pointer<DragEnd>>,
    mut action_manager: ResMut<ActionManager>,
    transforms: Query<&Transform>,
    mut commands: Commands,
    camera_query: Single<(&Camera, &GlobalTransform, &Projection)>,
) {
    if let Ok(transform) = transforms.get(drag.target()) {
        let (_, _, camera_projection) = *camera_query;
        let Projection::Orthographic(ortho) = camera_projection else {
            return;
        };

        let to_transform = *transform;
        let mut from_transform = to_transform.clone();
        let distance_delta = drag.distance * ortho.scale;
        from_transform.translation -= Vec3::new(distance_delta.x, -distance_delta.y, 0.0);
        let move_action = Box::new(MoveAction {
            entity: drag.target(),
            from: from_transform,
            to: to_transform,
        });
        action_manager.execute(move_action, &mut commands);
    }
}

fn drag_update(
    mut transforms: Query<&mut Transform, With<Selected>>,
    camera_query: Single<(&Camera, &GlobalTransform)>,
    key_input: Res<ButtonInput<KeyCode>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    window: Query<&Window>,
    time: Res<Time>,
) {
    let Some(cursor_position) = window.single().unwrap().cursor_position() else {
        return;
    };

    if mouse_input.pressed(MouseButton::Left) {
        for mut transform in transforms.iter_mut() {
            transform.translation = Vec3::new(cursor_position.x, cursor_position.y, 0.0);
            let (camera, camera_transform) = *camera_query;
            if let Ok(cursor_world) =
                camera.viewport_to_world_2d(camera_transform, cursor_position)
            {
                transform.translation = Vec3::new(cursor_world.x, cursor_world.y, 0.0);
            }
        }
    }

    // rotate
    if key_input.pressed(KeyCode::KeyE) {
        for mut transform in transforms.iter_mut() {
            transform.rotate_z((-100.0 * time.delta_secs()).to_radians());
        }
    }
    if key_input.pressed(KeyCode::KeyQ) {
        for mut transform in transforms.iter_mut() {
            transform.rotate_z((100.0 * time.delta_secs()).to_radians());
        }
    }
}

#[derive(Component)]
struct Selected;

fn selected_update(drag: Trigger<Pointer<Pressed>>, mut commands: Commands) {
    commands.entity(drag.target).insert(Selected);
}

fn unselected_update(drag: Trigger<Pointer<Released>>, mut commands: Commands) {
    commands.entity(drag.target).remove::<Selected>();
}
