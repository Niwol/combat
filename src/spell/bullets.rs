use std::f32::consts::PI;

use avian2d::prelude::{LinearDamping, LinearVelocity, RigidBody};
use bevy::prelude::*;

use crate::{
    health::Health,
    living_entity::{FacingDirection, enemy::Enemy},
};

use super::hitbox::{HitEntityEvent, HitboxInfo, SpawnHitboxEvent};

const BULLET_SIZE: Vec2 = Vec2 { x: 16.0, y: 8.0 };
const BULLET_DAMAGE: i32 = 1;

pub fn plugin(app: &mut App) {
    app.add_event::<CastBulletsSpell>();

    app.add_observer(cast_bullets_observer);

    app.add_systems(Startup, load_bullet_sprite);

    app.add_systems(Update, update_bullet_casting);
    app.add_systems(Update, update_and_despawns_bullets);
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Event)]
pub struct CastBulletsSpell;

#[derive(Resource)]
struct BulletSprites {
    bullet: Handle<Image>,
}

#[derive(Component)]
struct CastingBulletsComponent {
    nb_bullets_left: u32,
    bullet_timer: Timer,
    fire_direction: Vec2,
}

#[derive(Component)]
struct Bullet {
    live_timer: Timer,
}

fn load_bullet_sprite(mut commands: Commands, assets: Res<AssetServer>) {
    let bullet_sprites = BulletSprites {
        bullet: assets.load("bullet.png"),
    };

    commands.insert_resource(bullet_sprites);
}

fn cast_bullets_observer(
    trigger: Trigger<CastBulletsSpell>,
    mut commands: Commands,
    facing_direction: Query<&FacingDirection>,
) {
    let caster = trigger.target();

    if caster == Entity::PLACEHOLDER {
        println!("Bullets casted by unkown entity");
        return;
    }

    let fire_direction = facing_direction.get(caster);
    let fire_direction = fire_direction.unwrap_or(&FacingDirection(Vec2::ZERO)).0;

    let bullet_caster = commands
        .spawn((
            CastingBulletsComponent {
                nb_bullets_left: 20,
                bullet_timer: Timer::from_seconds(0.03, TimerMode::Repeating),
                fire_direction,
            },
            Transform::default(),
        ))
        .id();

    commands.entity(caster).add_child(bullet_caster);
}

fn update_bullet_casting(
    time: Res<Time>,
    mut commands: Commands,
    bullet_sprites: Res<BulletSprites>,
    mut caster: Query<(Entity, &mut CastingBulletsComponent, &GlobalTransform)>,
) {
    for (casting_entity, mut casting_bullets, global_trasform) in &mut caster {
        if casting_bullets
            .bullet_timer
            .tick(time.delta())
            .just_finished()
        {
            casting_bullets.nb_bullets_left -= 1;

            let angle = {
                if casting_bullets.fire_direction == Vec2::ZERO {
                    rand::random::<f32>() * 2.0 * PI
                } else {
                    let angle = Vec2::X.angle_to(casting_bullets.fire_direction);
                    let angle = angle + (rand::random::<f32>() - 0.5) * 0.5;
                    angle
                }
            };

            let bullet = commands
                .spawn((
                    Sprite {
                        image: bullet_sprites.bullet.clone(),
                        ..Default::default()
                    },
                    RigidBody::Dynamic,
                    LinearVelocity(Vec2::from_angle(angle) * 300.0),
                    LinearDamping(-0.9),
                    Transform::from_translation(global_trasform.translation())
                        .with_rotation(Quat::from_rotation_z(angle)),
                    Bullet {
                        live_timer: Timer::from_seconds(5.0, TimerMode::Once),
                    },
                ))
                .observe(resolve_enemy_hit)
                .id();

            commands.trigger(SpawnHitboxEvent(HitboxInfo {
                size: BULLET_SIZE,
                parent: bullet,
                live_timer: None,
            }));
        }

        if casting_bullets.nb_bullets_left == 0 {
            commands.entity(casting_entity).despawn();
        }
    }
}

fn update_and_despawns_bullets(
    mut commands: Commands,
    time: Res<Time>,
    mut bullets: Query<(Entity, &mut Bullet)>,
) {
    let delta = time.delta();

    for (entity, mut bullet) in &mut bullets {
        bullet.live_timer.tick(delta);

        if bullet.live_timer.finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn resolve_enemy_hit(
    trigger: Trigger<HitEntityEvent>,
    mut commands: Commands,
    mut enemies: Query<&mut Health, With<Enemy>>,
) {
    if let Ok(mut enemy_health) = enemies.get_mut(trigger.0) {
        enemy_health.0 -= BULLET_DAMAGE;

        commands.entity(trigger.target()).despawn();
    }
}
