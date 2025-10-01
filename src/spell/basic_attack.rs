use avian2d::prelude::LinearVelocity;
use bevy::prelude::*;

use crate::{health::Health, living_entity::FacingDirection};

use super::hitbox::{HitEntityEvent, HitboxInfo, SpawnHitboxEvent};

const ATTACK_DAMAGE: i32 = 4;

pub fn plugin(app: &mut App) {
    app.add_event::<CastBasicAttack>();

    app.add_observer(spawn_basic_attack);

    app.add_systems(Update, update_attack);
}

#[derive(Event)]
pub struct CastBasicAttack;

#[derive(Component)]
struct BasicAttack {
    live_timer: Timer,
    push_direction: Vec2,
}

fn spawn_basic_attack(
    trigger: Trigger<CastBasicAttack>,
    mut commands: Commands,
    transforms: Query<Option<&FacingDirection>>,
    assets: Res<AssetServer>,
) {
    let facing_direction = transforms.get(trigger.target()).unwrap();

    let mut transform = Transform::default();
    let mut push_direction = Vec2::ZERO;
    if let Some(facing_direction) = facing_direction {
        transform.translation += Vec3::from((facing_direction.0 * 32.0, 0.0));

        let angle = Vec2::X.angle_to(facing_direction.0);
        transform.rotate_z(angle);

        push_direction = facing_direction.0;
    }

    let attack = commands
        .spawn((
            Sprite {
                image: assets.load("slash.png"),
                rect: Some(Rect::new(0.0, 0.0, 32.0, 32.0)),
                ..Default::default()
            },
            BasicAttack {
                live_timer: Timer::from_seconds(0.05, TimerMode::Repeating),
                push_direction,
            },
            transform,
        ))
        .observe(hit_enemy)
        .id();

    commands.entity(trigger.target()).add_child(attack);

    commands.trigger(SpawnHitboxEvent(HitboxInfo {
        size: Vec2::splat(32.0),
        parent: attack,
        live_timer: None,
    }));
}

fn update_attack(
    time: Res<Time>,
    mut commands: Commands,
    mut attacks: Query<(Entity, &mut BasicAttack, &mut Sprite)>,
) {
    let delta = time.delta();

    for (entity, mut attack, mut sprite) in &mut attacks {
        attack.live_timer.tick(delta);

        if attack.live_timer.just_finished() {
            let mut rect = sprite.rect.unwrap();
            rect.min.x += 32.0;
            rect.max.x += 32.0;

            sprite.rect = Some(rect);

            if rect.max.x > 32.0 * 4.0 {
                commands.entity(entity).despawn();
            }
        }
    }
}

fn hit_enemy(
    trigger: Trigger<HitEntityEvent>,
    attacks: Query<&BasicAttack>,
    mut enemies: Query<(&mut Health, &mut LinearVelocity)>,
) {
    let enemy = trigger.0;
    let attack = attacks.get(trigger.target()).unwrap();

    match enemies.get_mut(enemy) {
        Ok((mut enemy_health, mut linear_velocity)) => {
            enemy_health.0 -= ATTACK_DAMAGE;
            linear_velocity.0 += attack.push_direction * 30.0;
        }
        Err(_) => (),
    };
}
