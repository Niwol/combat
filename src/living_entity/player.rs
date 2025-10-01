use std::f32::consts::PI;

use avian2d::prelude::*;
use bevy::{color::palettes, prelude::*};

use crate::{
    character_controller::CharacterController,
    interaction::Interactor,
    living_entity::{EntityController, EntityState, EntityStats, FacingDirection},
    skills::{Skill, SkillTree},
    xp::XpInventory,
};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_player);
        app.add_systems(Update, direction_change);
        app.add_systems(Update, show_player_view);
    }
}

#[derive(Debug, Component, Copy, Clone)]
pub struct Player;

#[derive(Component, Clone, Copy)]
pub enum Eye {
    Left,
    Right,
}

#[derive(Component)]
pub struct PlayerInteractor;

fn show_player_view(
    mut gizoms: Gizmos,
    player: Single<(&Transform, &FacingDirection), With<Player>>,
) {
    gizoms.arc_2d(
        Isometry2d {
            rotation: Rot2::radians(player.1.0.to_angle() - PI / 2.0 - PI / 3.0),
            translation: player.0.translation.xy(),
        },
        PI / 1.5,
        500.0,
        palettes::basic::GREEN,
    );
}

fn player_skill_tree() -> SkillTree {
    let mut skill_tree = SkillTree::new();

    skill_tree.add_skill(Skill {
        name: String::from("Slash"),
        mana_cost: 0,
        reload_timer: Timer::from_seconds(0.2, TimerMode::Once),
        unlocked: true,
    });

    skill_tree.add_skill(Skill {
        name: String::from("Bullets"),
        mana_cost: 20,
        reload_timer: Timer::from_seconds(1.0, TimerMode::Once),
        unlocked: false,
    });

    skill_tree.add_skill(Skill {
        name: String::from("Fire ball"),
        mana_cost: 50,
        reload_timer: Timer::from_seconds(2.0, TimerMode::Once),
        unlocked: false,
    });

    skill_tree.add_skill(Skill {
        name: String::from("Beam"),
        mana_cost: 10,
        reload_timer: Timer::from_seconds(3.0, TimerMode::Once),
        unlocked: false,
    });

    skill_tree
}

fn spawn_player(mut commands: Commands) {
    let facing_direction = FacingDirection(Vec2::X);

    let eye_position_1 = facing_direction.0 * 8.0 + facing_direction.0.perp() * 3.0;
    let eye_position_2 = facing_direction.0 * 8.0 - facing_direction.0.perp() * 3.0;

    commands.spawn((
        Sprite {
            color: palettes::basic::AQUA.into(),
            custom_size: Some(Vec2::splat(16.0)),
            ..Default::default()
        },
        CharacterController {
            current_speed: 0.0,
            max_speed: 300.0,
        },
        EntityController {
            state: EntityState::Idle,
            stats: EntityStats { max_speed: 300.0 },
            ..Default::default()
        },
        RigidBody::Kinematic,
        facing_direction,
        Player,
        {
            let mut xp = XpInventory::new();
            xp.collect_xp(2000);
            xp
        },
        player_skill_tree(),
        children![
            (
                Sprite {
                    color: palettes::basic::RED.into(),
                    custom_size: Some(Vec2::splat(3.0)),
                    ..Default::default()
                },
                Transform::from_translation(Vec3::from((eye_position_1, 1.0))),
                Eye::Left,
            ),
            (
                Sprite {
                    color: palettes::basic::RED.into(),
                    custom_size: Some(Vec2::splat(3.0)),
                    ..Default::default()
                },
                Transform::from_translation(Vec3::from((eye_position_2, 1.0))),
                Eye::Right,
            ),
            (
                Collider::rectangle(5.0, 5.0),
                Sensor,
                Interactor,
                PlayerInteractor,
                Transform::from_xyz(20.0, 0.0, 0.0)
            )
        ],
    ));
}

fn direction_change(
    facing_direction: Query<&FacingDirection, (With<Player>, Changed<FacingDirection>)>,
    mut eyes: Query<(&mut Transform, &Eye, &ChildOf), Without<Interactor>>,
    mut interactor: Query<(&mut Transform, &ChildOf), With<Interactor>>,
) {
    for (mut transform, eye, child_of) in &mut eyes {
        if let Ok(facing_direction) = facing_direction.get(child_of.0) {
            let eye_direction = match eye {
                Eye::Left => facing_direction.0.perp(),
                Eye::Right => -facing_direction.0.perp(),
            };

            transform.translation =
                Vec3::from((facing_direction.0 * 8.0 + eye_direction * 3.0, 1.0));
        }
    }

    for (mut transform, child_of) in &mut interactor {
        if let Ok(facing_direction) = facing_direction.get(child_of.0) {
            let dir = facing_direction.0;

            transform.translation = Vec3::from((dir * 20.0, 0.0));
        }
    }
}
