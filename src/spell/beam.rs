use bevy::prelude::*;

use crate::{health::Health, living_entity::FacingDirection};

use super::hitbox::{HitEntityEvent, HitboxInfo, SpawnHitboxEvent};

const BEAM_LENGTH: f32 = 50.0;
const BEAM_DAMAGE: i32 = 1;

pub fn plugin(app: &mut App) {
    app.add_event::<CastBeamSpell>();

    app.add_systems(Startup, load_beam_sprites);

    app.add_observer(cast_beam);

    app.add_systems(Update, update_beams);
}

#[derive(Event)]
pub struct CastBeamSpell;

#[derive(Resource)]
struct BeamSprites {
    beam: Handle<Image>,
}

#[derive(Component)]
struct Beam {
    live_timer: Timer,
    hitbox_timer: Timer,
    frame_timer: Timer,
}

fn load_beam_sprites(mut commands: Commands, assets: Res<AssetServer>) {
    let beam_sprites = BeamSprites {
        beam: assets.load("beam.png"),
    };

    commands.insert_resource(beam_sprites);
}

fn cast_beam(
    trigger: Trigger<CastBeamSpell>,
    mut commands: Commands,
    beam_sprites: Res<BeamSprites>,
    facing_direction: Query<&FacingDirection>,
) {
    let mut direction = Vec2::X;
    if let Ok(facing_direction) = facing_direction.get(trigger.target()) {
        direction = facing_direction.0;
    }

    let angle = Vec2::X.angle_to(direction);

    let beam = commands
        .spawn((
            Sprite {
                custom_size: Some(Vec2 {
                    x: 16.0 * BEAM_LENGTH,
                    y: 16.0,
                }),
                image: beam_sprites.beam.clone(),
                rect: Some(Rect::new(0.0, 0.0, 16.0, 16.0)),
                image_mode: SpriteImageMode::Tiled {
                    tile_x: true,
                    tile_y: true,
                    stretch_value: 1.0,
                },
                ..Default::default()
            },
            Beam {
                live_timer: Timer::from_seconds(2.0, TimerMode::Once),
                hitbox_timer: Timer::from_seconds(0.2, TimerMode::Repeating),
                frame_timer: Timer::from_seconds(0.1, TimerMode::Repeating),
            },
            Transform::from_rotation(Quat::from_rotation_z(angle))
                .with_translation(Vec3::from((direction * 16.0 * BEAM_LENGTH / 2.0, 0.0))),
        ))
        .observe(resolve_enemy_hit)
        .id();

    commands.entity(trigger.target()).add_child(beam);
}

fn update_beams(
    mut commands: Commands,
    time: Res<Time>,
    mut beams: Query<(Entity, &mut Beam, &mut Sprite)>,
) {
    let delta = time.delta();
    for (entity, mut beam, mut sprite) in &mut beams {
        beam.live_timer.tick(delta);
        beam.hitbox_timer.tick(delta);
        beam.frame_timer.tick(delta);

        if beam.frame_timer.just_finished() {
            let mut rect = sprite.rect.unwrap();

            rect.min.x += 16.0;
            rect.max.x += 16.0;

            if rect.max.x > 16.0 * 3.0 {
                rect.min.x = 0.0;
                rect.max.x = 16.0;
            }

            sprite.rect = Some(rect);
        }

        if beam.hitbox_timer.just_finished() {
            commands.trigger(SpawnHitboxEvent(HitboxInfo {
                size: sprite.custom_size.unwrap(),
                parent: entity,
                live_timer: Some(Timer::from_seconds(0.2, TimerMode::Once)),
            }));
        }

        if beam.live_timer.finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn resolve_enemy_hit(trigger: Trigger<HitEntityEvent>, mut entities: Query<&mut Health>) {
    if let Ok(mut health) = entities.get_mut(trigger.0) {
        health.0 -= BEAM_DAMAGE;
    }
}
