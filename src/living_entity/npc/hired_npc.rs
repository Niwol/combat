use bevy::prelude::*;

use crate::{
    head_quarter::HeadQuarter,
    living_entity::{
        EntityController, EntityState,
        npc::{DecisionMaker, NPCState},
    },
};

pub fn plugin(app: &mut App) {
    app.add_systems(Update, update_hired_npcs);
}

#[derive(Component)]
pub struct HiredNPC;

fn update_hired_npcs(
    time: Res<Time>,
    head_quarter: Single<&GlobalTransform, With<HeadQuarter>>,
    mut decision_makers: Query<
        (&mut DecisionMaker, &GlobalTransform, &mut EntityController),
        With<HiredNPC>,
    >,
) {
    for (mut decision_maker, global_transform, mut entity_controller) in &mut decision_makers {
        match decision_maker.state {
            NPCState::Idle => {
                if decision_maker.decision_timer.tick(time.delta()).finished() {
                    let x = (rand::random::<f32>() - 0.5) * 100.0;
                    let y = (rand::random::<f32>() - 0.5) * 100.0;
                    let pos = head_quarter.translation();

                    let target = Vec2 {
                        x: pos.x + x,
                        y: pos.y + y,
                    };
                    decision_maker.state = NPCState::MoveTo { target }
                }
            }

            NPCState::MoveTo { target } => {
                let pos = global_transform.translation();
                let to_target = target - pos.xy();

                if to_target.length() > 3.0 {
                    entity_controller.state = EntityState::Move {
                        direction: to_target.normalize(),
                    };
                } else {
                    entity_controller.state = EntityState::Idle;
                    decision_maker.decision_timer =
                        Timer::from_seconds(rand::random_range(5..=10) as f32, TimerMode::Once);
                    decision_maker.state = NPCState::Idle;
                }
            }

            NPCState::Interacting => (),
        }
    }
}
