use bevy::prelude::*;

use crate::living_entity::player::Player;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera);
        app.add_systems(Update, update_camera_position);
    }
}

#[derive(Component)]
struct MainCamera;

fn spawn_camera(mut commands: Commands) {
    let mut orthographic_projection = OrthographicProjection::default_2d();
    orthographic_projection.scale = 1.0 / 4.0;

    let projection = Projection::Orthographic(orthographic_projection);

    commands.spawn((Camera2d, projection, MainCamera));
}

fn update_camera_position(
    mut camera: Single<&mut Transform, With<MainCamera>>,
    player: Single<&Transform, (With<Player>, Without<MainCamera>)>,
) {
    let rect_size = 100.0 / 4.0;

    if camera.translation.x < player.translation.x - rect_size {
        camera.translation.x = player.translation.x - rect_size;
    }
    if camera.translation.x > player.translation.x + rect_size {
        camera.translation.x = player.translation.x + rect_size;
    }
    if camera.translation.y < player.translation.y - rect_size {
        camera.translation.y = player.translation.y - rect_size;
    }
    if camera.translation.y > player.translation.y + rect_size {
        camera.translation.y = player.translation.y + rect_size;
    }
}
