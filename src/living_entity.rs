use avian2d::prelude::LinearVelocity;
use bevy::prelude::*;

use crate::living_entity::{
    character::CharacterPlugin, enemy::EnemyPlugin, npc::NPCPlugin, player::PlayerPlugin,
};

pub mod character;
pub mod enemy;
pub mod npc;
pub mod player;

pub struct LivingEntityPlugin;

impl Plugin for LivingEntityPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((CharacterPlugin, PlayerPlugin, EnemyPlugin, NPCPlugin));

        app.add_systems(Update, controll_entities);
    }
}

#[derive(Component)]
#[require(Team)]
pub struct LivingEntity;

#[derive(Component, Default)]
pub enum Team {
    #[default]
    Neutral,
    Ally,
    Enemy,
}

#[derive(Component, Clone, Copy)]
pub struct FacingDirection(pub Vec2);

#[derive(Component)]
pub struct EntityController {
    pub state: EntityState,
    pub stats: EntityStats,
    inner_stats: EntityInnerStats,
}

impl Default for EntityController {
    fn default() -> Self {
        Self {
            state: EntityState::Idle,
            stats: EntityStats { max_speed: 50.0 },
            inner_stats: EntityInnerStats::default(),
        }
    }
}

#[derive(Default)]
pub enum EntityState {
    #[default]
    Idle,
    Move {
        direction: Vec2,
    },
}

pub struct EntityStats {
    pub max_speed: f32,
}

struct EntityInnerStats {
    acceleration_timer: Timer,
}

impl Default for EntityInnerStats {
    fn default() -> Self {
        Self {
            acceleration_timer: Timer::from_seconds(0.2, TimerMode::Once),
        }
    }
}

fn controll_entities(
    time: Res<Time>,
    mut entities: Query<(
        &mut EntityController,
        &mut LinearVelocity,
        Option<&mut FacingDirection>,
    )>,
) {
    let dt = time.delta();

    for (mut controller, mut velocity, facing_direction) in &mut entities {
        match controller.state {
            EntityState::Idle => {
                velocity.0 = Vec2::ZERO;
                controller.inner_stats.acceleration_timer.reset();
            }
            EntityState::Move { direction } => {
                let timer = &mut controller.inner_stats.acceleration_timer;
                let progress = timer.tick(dt).elapsed_secs();
                let duration = timer.duration();

                let progress = progress / duration.as_secs_f32();

                let acc = easing::EaseFunction::SineOut.sample(progress).unwrap();

                velocity.0 = direction * controller.stats.max_speed * acc;
                if let Some(mut facing_direction) = facing_direction {
                    facing_direction.0 = direction;
                }
            }
        }
    }
}
