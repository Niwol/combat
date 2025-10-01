use std::time::Duration;

use bevy::prelude::*;

use crate::living_entity::enemy::SpawnEnemyEvent;

pub struct EnemySpawnerPlugin;

impl Plugin for EnemySpawnerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnEnemySpawnerEvent>();

        app.add_systems(
            Startup,
            (load_sprites, spawn_initial_enemy_spawners).chain(),
        );
        app.add_systems(Update, spawn_enemy);

        app.add_observer(spawn_enemy_spawner);
    }
}

#[derive(Resource)]
struct Sprites {
    spawn_hole: Handle<Image>,
}

#[derive(Event)]
struct SpawnEnemySpawnerEvent(Vec2);

#[derive(Component)]
struct EnemySpawner {
    spawning_rate: Timer,
}

fn load_sprites(mut commands: Commands, assets: Res<AssetServer>) {
    let sprites = Sprites {
        spawn_hole: assets.load("spawn_hole.png"),
    };

    commands.insert_resource(sprites);
}

fn spawn_initial_enemy_spawners(mut commands: Commands) {
    for x in 0..4 {
        for y in 0..4 {
            let r1 = rand::random::<f32>();
            let r2 = rand::random::<f32>();

            let pos = Vec2 {
                x: 1000.0 + x as f32 * 200.0 + r1 * 100.0,
                y: -400.0 + y as f32 * 200.0 + r2 * 100.0,
            };

            commands.trigger(SpawnEnemySpawnerEvent(pos));
        }
    }
}

fn spawn_enemy_spawner(
    trigger: Trigger<SpawnEnemySpawnerEvent>,
    mut commands: Commands,
    sprites: Res<Sprites>,
) {
    let x = trigger.0.x;
    let y = trigger.0.y;

    commands.spawn((
        Sprite {
            image: sprites.spawn_hole.clone(),
            ..Default::default()
        },
        Transform::from_xyz(x, y, 0.0),
        EnemySpawner {
            spawning_rate: Timer::from_seconds(2.0, TimerMode::Repeating),
        },
    ));
}

fn spawn_enemy(
    mut commands: Commands,
    time: Res<Time>,
    mut enemy_spawners: Query<(&mut EnemySpawner, &Transform)>,
) {
    let dt = time.delta();

    let offset = Vec2 {
        x: (rand::random::<f32>() - 0.5) * 100.0,
        y: (rand::random::<f32>() - 0.5) * 100.0,
    };

    for (mut enemy_spawner, transform) in &mut enemy_spawners {
        if enemy_spawner.spawning_rate.tick(dt).just_finished() {
            commands.trigger(SpawnEnemyEvent(transform.translation.xy() + offset));

            enemy_spawner
                .spawning_rate
                .set_duration(Duration::from_secs_f32(2.0 + rand::random::<f32>() * 0.5));
        }
    }
}
