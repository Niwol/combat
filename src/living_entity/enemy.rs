use std::f32::consts::PI;

use avian2d::prelude::{Collider, CollisionEventsEnabled, LinearVelocity, LockedAxes, RigidBody};
use bevy::{color::palettes, prelude::*};

use crate::{
    health::Health,
    living_entity::{EntityController, EntityState, EntityStats, FacingDirection, player::Player},
    xp::SpawnXpEvent,
};

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnEnemyEvent>();

        app.add_systems(Startup, load_enemy_sprites);

        app.add_observer(spawn_enemy);

        app.add_systems(Update, (update_enemies, update_enemy_sprites));
        app.add_systems(PostUpdate, despawn_dead_enemies);

        app.add_systems(Update, show_closest_enemy);
    }
}

#[derive(Event)]
pub struct SpawnEnemyEvent(pub Vec2);

#[derive(Resource)]
struct EnemySprites {
    diablo: Handle<Image>,
}

#[derive(Debug, Component, Clone)]
pub struct Enemy {
    frame_timer: Timer,
}

fn load_enemy_sprites(mut commands: Commands, assets: Res<AssetServer>) {
    let enemy_sprites = EnemySprites {
        diablo: assets.load("enemy.png"),
    };

    commands.insert_resource(enemy_sprites);
}

fn spawn_enemy(
    trigger: Trigger<SpawnEnemyEvent>,
    mut commands: Commands,
    enemy_sprites: Res<EnemySprites>,
) {
    let x = trigger.0.x;
    let y = trigger.0.y;

    commands.spawn((
        Sprite {
            image: enemy_sprites.diablo.clone(),
            rect: Some(Rect::new(0.0, 0.0, 16.0, 16.0)),
            ..Default::default()
        },
        RigidBody::Dynamic,
        Collider::circle(8.0),
        CollisionEventsEnabled,
        LockedAxes::ROTATION_LOCKED,
        Enemy {
            frame_timer: Timer::from_seconds(0.15, TimerMode::Repeating),
        },
        Transform::from_xyz(x, y, 0.0),
        Health(10),
        EntityController {
            state: EntityState::Idle,
            stats: EntityStats { max_speed: 40.0 },
            ..Default::default()
        },
    ));
}

fn update_enemies(
    mut enemies: Query<(&mut EntityController, &Transform), With<Enemy>>,
    player: Single<&Transform, With<Player>>,
) {
    for (mut entity_controller, enemy_transform) in &mut enemies {
        let to_player = (player.translation - enemy_transform.translation).xy();

        let view_dist = 700.0;

        if to_player.length() < view_dist {
            entity_controller.state = EntityState::Move {
                direction: to_player.normalize(),
            }
        } else {
            entity_controller.state = EntityState::Idle;
        }
    }
}

fn update_enemy_sprites(
    time: Res<Time>,
    mut enemies: Query<(&mut Enemy, &mut Sprite, &LinearVelocity)>,
) {
    let delta = time.delta();

    for (mut enemy, mut sprite, velocity) in &mut enemies {
        if velocity.0 == Vec2::ZERO {
            sprite.rect = Some(Rect::new(0.0, 0.0, 16.0, 16.0));
            enemy.frame_timer.reset();
            continue;
        }

        enemy.frame_timer.tick(delta);

        if enemy.frame_timer.just_finished() {
            let mut rect = sprite.rect.unwrap();

            rect.min.x += 16.0;
            rect.max.x += 16.0;

            if rect.max.x > 16.0 * 6.0 {
                rect.min.x = 0.0;
                rect.max.x = 16.0;
            }

            sprite.rect = Some(rect);
        }

        // flip sprite
        if velocity.0.x < 0.0 {
            sprite.flip_x = true;
        } else if velocity.0.x > 0.0 {
            sprite.flip_x = false;
        }
    }
}

fn despawn_dead_enemies(
    mut commands: Commands,
    enemies: Query<(Entity, &Health, &Transform), With<Enemy>>,
) {
    for (entity, health, transform) in &enemies {
        if health.0 <= 0 {
            commands.entity(entity).despawn();

            commands.trigger(SpawnXpEvent {
                location: transform.translation.xy(),
                amount: 10,
            });
        }
    }
}

fn show_closest_enemy(
    mut gizmos: Gizmos,
    enemies: Query<&Transform, With<Enemy>>,
    player: Single<(&Transform, &FacingDirection), With<Player>>,
) {
    let enemies = enemies.into_iter().filter(|enemy| {
        let to_enemy = enemy.translation.xy() - player.0.translation.xy();

        let view_angle = PI / 3.0;

        let enemy_angle = to_enemy.angle_to(player.1.0).abs();

        view_angle > enemy_angle && to_enemy.length() < 500.0
    });

    let enemy = enemies.min_by(|enemy_1, enemy_2| {
        let dist_1 = (enemy_1.translation - player.0.translation).length();
        let dist_2 = (enemy_2.translation - player.0.translation).length();

        dist_1.partial_cmp(&dist_2).unwrap()
    });

    if let Some(enemy) = enemy {
        gizmos.rect_2d(
            Isometry2d::from_translation(enemy.translation.xy()),
            Vec2::splat(13.0),
            palettes::basic::FUCHSIA,
        );
    }
}
