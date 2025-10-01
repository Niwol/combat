use std::f32::consts::PI;

use avian2d::prelude::{LinearDamping, LinearVelocity, RigidBody};
use bevy::prelude::*;

use crate::{health::Health, living_entity::FacingDirection};

use super::hitbox::{HitEntityEvent, HitboxInfo, SpawnHitboxEvent};

const FIRE_BALL_DAMAGE: i32 = 5;
const SMALL_FIRE_BALL_DAMAGE: i32 = 1;

pub fn plugin(app: &mut App) {
    app.add_event::<CastFireBallSpell>();
    app.add_event::<SpawnSmallFireBallEvent>();

    app.add_systems(Startup, load_fire_ball_sprites);
    app.add_systems(Update, (update_fire_balls, update_small_fire_balls));

    app.add_observer(spawn_fire_ball);
    app.add_observer(spawn_small_fire_ball);
}

#[derive(Event)]
pub struct CastFireBallSpell;

#[derive(Event)]
struct SpawnSmallFireBallEvent;

#[derive(Resource)]
struct FireBallSprites {
    big: Handle<Image>,
    small: Handle<Image>,
}

#[derive(Component)]
struct FireBall {
    live_timer: Timer,
    frame_timer: Timer,
    spawn_timer: Timer,
}

#[derive(Component)]
struct SmallFireBall {
    live_timer: Timer,
    frame_timer: Timer,
}

fn load_fire_ball_sprites(mut commands: Commands, asstes: Res<AssetServer>) {
    let fire_ball_sprites = FireBallSprites {
        big: asstes.load("fire_ball.png"),
        small: asstes.load("small_fire_ball.png"),
    };

    commands.insert_resource(fire_ball_sprites);
}

fn spawn_fire_ball(
    trigger: Trigger<CastFireBallSpell>,
    mut commands: Commands,
    caster: Query<(&Transform, Option<&FacingDirection>)>,
    fire_ball_sprites: Res<FireBallSprites>,
) {
    let (transform, facing_direction) = caster.get(trigger.target()).unwrap();

    let mut velocity = Vec2::X;
    if let Some(facing_direction) = facing_direction {
        velocity = facing_direction.0;
    }

    velocity *= 50.0;

    let fire_ball = commands
        .spawn((
            Sprite {
                image: fire_ball_sprites.big.clone(),
                rect: Some(Rect::new(0.0, 0.0, 32.0, 32.0)),
                ..Default::default()
            },
            *transform,
            RigidBody::Dynamic,
            LinearVelocity(velocity),
            FireBall {
                live_timer: Timer::from_seconds(10.0, TimerMode::Once),
                frame_timer: Timer::from_seconds(0.2, TimerMode::Repeating),
                spawn_timer: Timer::from_seconds(0.02, TimerMode::Repeating),
            },
        ))
        .observe(resolve_enemy_hit)
        .id();

    commands.trigger(SpawnHitboxEvent(HitboxInfo {
        size: Vec2::splat(32.0),
        parent: fire_ball,
        live_timer: None,
    }));
}

fn update_fire_balls(
    mut commands: Commands,
    time: Res<Time>,
    mut fire_balls: Query<(Entity, &mut FireBall, &mut Sprite)>,
) {
    let delta = time.delta();
    for (entity, mut fire_ball, mut sprite) in &mut fire_balls {
        fire_ball.live_timer.tick(delta);
        fire_ball.frame_timer.tick(delta);
        fire_ball.spawn_timer.tick(delta);

        if fire_ball.frame_timer.just_finished() {
            let mut rect = sprite.rect.unwrap();

            rect.min.x += 32.0;
            rect.max.x += 32.0;
            if rect.max.x > 32.0 * 7.0 {
                rect.min.x = 0.0;
                rect.max.x = 32.0;
            }

            sprite.rect = Some(rect);
        }

        if fire_ball.spawn_timer.just_finished() {
            commands.trigger_targets(SpawnSmallFireBallEvent, entity);
        }

        if fire_ball.live_timer.finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn update_small_fire_balls(
    mut commands: Commands,
    time: Res<Time>,
    mut small_fire_balls: Query<(Entity, &mut SmallFireBall, &mut Sprite)>,
) {
    let delta = time.delta();
    for (entity, mut small_fire_ball, mut sprite) in &mut small_fire_balls {
        small_fire_ball.live_timer.tick(delta);
        small_fire_ball.frame_timer.tick(delta);

        if small_fire_ball.frame_timer.just_finished() {
            let mut rect = sprite.rect.unwrap();

            rect.min.x += 8.0;
            rect.max.x += 8.0;
            if rect.max.x > 8.0 * 4.0 {
                rect.min.x = 0.0;
                rect.max.x = 8.0;
            }

            sprite.rect = Some(rect);
        }

        if small_fire_ball.live_timer.finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn spawn_small_fire_ball(
    trigger: Trigger<SpawnSmallFireBallEvent>,
    mut commands: Commands,
    fire_ball_sprites: Res<FireBallSprites>,
    fire_balls: Query<&Transform>,
) {
    let transform = fire_balls.get(trigger.target()).unwrap();

    let r = rand::random::<f32>() * 2.0 * PI;

    let x = f32::cos(r);
    let y = f32::sin(r);

    let velocity = Vec2 { x, y } * 200.0;

    let small_fire_ball = commands
        .spawn((
            Sprite {
                image: fire_ball_sprites.small.clone(),
                rect: Some(Rect::new(0.0, 0.0, 8.0, 8.0)),
                ..Default::default()
            },
            *transform,
            RigidBody::Dynamic,
            LinearVelocity(velocity),
            LinearDamping(0.9999),
            SmallFireBall {
                live_timer: Timer::from_seconds(1.0, TimerMode::Once),
                frame_timer: Timer::from_seconds(0.05, TimerMode::Repeating),
            },
        ))
        .observe(resolve_small_fire_ball_hit)
        .id();

    commands.trigger(SpawnHitboxEvent(HitboxInfo {
        size: Vec2::splat(8.0),
        parent: small_fire_ball,
        live_timer: None,
    }));
}

fn resolve_enemy_hit(trigger: Trigger<HitEntityEvent>, mut entities: Query<&mut Health>) {
    if let Ok(mut health) = entities.get_mut(trigger.0) {
        health.0 -= FIRE_BALL_DAMAGE;
    }
}

fn resolve_small_fire_ball_hit(
    trigger: Trigger<HitEntityEvent>,
    mut commands: Commands,
    mut entities: Query<&mut Health>,
) {
    if let Ok(mut health) = entities.get_mut(trigger.0) {
        health.0 -= SMALL_FIRE_BALL_DAMAGE;

        commands.entity(trigger.target()).despawn();
    }
}
