use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

use combat::{
    camera::CameraPlugin, character_controller::CharacterControllerPlugin,
    enemy_spawner::EnemySpawnerPlugin, head_quarter::HeadQuarterPlugin, health::HealthPlugin,
    interaction::InteractionPlugin, living_entity::LivingEntityPlugin, skills::SkillPlugin,
    spell::SpellPlugin, ui::UiPlugin, xp::XpPlugin,
};

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()));
    app.add_plugins((
        PhysicsPlugins::default().set(PhysicsInterpolationPlugin::interpolate_all()),
        PhysicsDebugPlugin::default(),
    ));
    app.add_plugins((
        TiledPlugin::default(),
        TiledPhysicsPlugin::<TiledPhysicsAvianBackend>::default(),
    ));

    app.insert_resource(Gravity(Vec2::ZERO));

    app.add_plugins((
        LivingEntityPlugin,
        CharacterControllerPlugin,
        SpellPlugin,
        HealthPlugin,
        XpPlugin,
        EnemySpawnerPlugin,
        CameraPlugin,
        UiPlugin,
        HeadQuarterPlugin,
        InteractionPlugin,
        SkillPlugin,
    ));

    app.add_systems(Startup, setup);
    app.add_systems(Update, toggle_debug_view);

    app.run();
}

fn setup(mut commands: Commands, assets: Res<AssetServer>) {
    let handle = assets.load("maps/map.tmx");

    commands.spawn((
        TiledMap(handle),
        TilemapAnchor::Center,
        Transform::from_xyz(0.0, 100.0, -1.0).with_scale(Vec3::splat(1.0)),
    ));
}

fn toggle_debug_view(
    input: Res<ButtonInput<KeyCode>>,
    mut gizmos_config_store: ResMut<GizmoConfigStore>,
) {
    if input.just_pressed(KeyCode::F1) {
        let enabled = gizmos_config_store.config::<PhysicsGizmos>().0.enabled;
        gizmos_config_store.config_mut::<PhysicsGizmos>().0.enabled = !enabled;
    }
}
