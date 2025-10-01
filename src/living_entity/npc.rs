use std::fs;

use avian2d::prelude::{Collider, LinearVelocity, RigidBody};
use bevy::prelude::*;
use rand::random;

use crate::{
    character_controller::PlayerInputState,
    interaction::{Interactable, InteractionEvent},
    living_entity::{
        EntityController, EntityState, EntityStats, LivingEntity,
        character::{self, CharacterSprites},
        npc::hired_npc::HiredNPC,
        player::{Player, PlayerInteractor},
    },
    ui::ui_dialog::{
        DialogAction, DialogActionEvent, DialogButton, DialogButtons, DialogNode, DialogTree,
    },
    xp::XpInventory,
};

pub mod hired_npc;

pub struct NPCPlugin;

impl Plugin for NPCPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(hired_npc::plugin);

        app.add_event::<SpawnNPCEvent>();

        app.add_systems(Startup, spawn_initial_npcs.after(character::load_sprites));
        app.add_observer(spawn_npc);
        app.add_systems(Update, update_free_npcs);
    }
}

#[derive(Event)]
pub struct SpawnNPCEvent(Vec2);

#[derive(Component)]
struct NPC;

#[derive(Component)]
struct FreeNPC;

enum NPCState {
    Idle,
    MoveTo { target: Vec2 },
    Interacting,
}

#[derive(Component)]
struct DecisionMaker {
    decision_timer: Timer,
    state: NPCState,
}

fn spawn_initial_npcs(mut commands: Commands) {
    let nb_npcs = 20;
    for _ in 0..nb_npcs {
        let x = rand::random::<f32>() * 600.0 - 300.0;
        let y = rand::random::<f32>() * 300.0 + 200.0;

        commands.trigger(SpawnNPCEvent(Vec2 { x, y }));
    }
}

fn get_random_name() -> String {
    let names = fs::read_to_string("assets/names.txt").unwrap();

    let names = names.split("\n").collect::<Vec<_>>();

    let random_idx = rand::random_range(0..names.len());

    names[random_idx].to_string()
}

fn spawn_npc(
    trigger: Trigger<SpawnNPCEvent>,
    mut commands: Commands,
    sprites: Res<CharacterSprites>,
) {
    let x = rand::random_range(0..5) as f32;
    let y = rand::random_range(0..=1) as f32;

    let x0 = 16.0 + x * 16.0 * 3.0;
    let x1 = 16.0 + x * 16.0 * 3.0 + 16.0;
    let y0 = 0.0 + y * 16.0 * 4.0;
    let y1 = 0.0 + y * 16.0 * 4.0 + 16.0;

    let rect = Rect::new(x0, y0, x1, y1);
    let x = trigger.0.x;
    let y = trigger.0.y;

    commands
        .spawn((
            Name::new(get_random_name()),
            Sprite {
                image: sprites.generic.clone(),
                rect: Some(rect),
                ..Default::default()
            },
            Transform::from_xyz(x, y, 0.0),
            NPC,
            FreeNPC,
            DecisionMaker {
                decision_timer: Timer::from_seconds(5.0, TimerMode::Once),
                state: NPCState::Idle,
            },
            LivingEntity,
            EntityController {
                state: EntityState::Idle,
                stats: EntityStats { max_speed: 40.0 },
                ..Default::default()
            },
            RigidBody::Dynamic,
            LinearVelocity::default(),
            Interactable::new(),
            Collider::rectangle(20.0, 20.0),
        ))
        .observe(dialog_action)
        .observe(npc_interaction);
}

fn npc_interaction(
    trigger: Trigger<InteractionEvent>,
    mut commands: Commands,
    mut _player: Single<(Entity, &mut XpInventory), With<Player>>,
    player_interactor: Single<Entity, With<PlayerInteractor>>,
    mut npcs: Query<(Entity, &mut DecisionMaker), With<FreeNPC>>,
    mut next_player_state: ResMut<NextState<PlayerInputState>>,
) {
    if trigger.interactor != *player_interactor {
        return;
    }

    if let Ok((entity, mut decision_maker)) = npcs.get_mut(trigger.target()) {
        decision_maker.state = NPCState::Interacting;

        if trigger.interactor == *player_interactor {
            next_player_state.set(PlayerInputState::Menu);
        }

        let dialog_tree = DialogTree::new(DialogNode {
            text: String::from("Hello,\nHow can I help you ?"),
            action: DialogAction::Buttons(DialogButtons {
                selected_button: 0,
                buttons: vec![
                    DialogButton::new("Hire (500Xp)"),
                    DialogButton::new("Cancel"),
                ],
            }),
        });

        commands.entity(entity).insert(dialog_tree);
    }
}

fn dialog_action(
    trigger: Trigger<DialogActionEvent>,
    mut commands: Commands,
    dialog_trees: Query<&DialogTree>,
    mut next_player_state: ResMut<NextState<PlayerInputState>>,
    mut player: Single<&mut XpInventory, With<Player>>,
    mut npcs: Query<&mut DecisionMaker, With<FreeNPC>>,
) {
    let dialog_tree = dialog_trees.get(trigger.target()).unwrap();

    match *trigger {
        DialogActionEvent::Confirm => {
            let node = &dialog_tree.nodes[dialog_tree.current_node];

            if let DialogAction::Buttons(dialog_buttons) = &node.action {
                let button = &dialog_buttons.buttons[dialog_buttons.selected_button];

                match button.text.as_str() {
                    "Hire (500Xp)" => {
                        if player.spend(500) {
                            commands
                                .entity(trigger.target())
                                .remove::<FreeNPC>()
                                .remove::<DialogTree>()
                                .insert(HiredNPC);

                            next_player_state.set(PlayerInputState::CharacterController);
                            let mut npc = npcs.get_mut(trigger.target()).unwrap();
                            npc.state = NPCState::Idle;
                        }
                    }
                    "Cancel" => {
                        commands.entity(trigger.target()).remove::<DialogTree>();
                        next_player_state.set(PlayerInputState::CharacterController);
                        let mut npc = npcs.get_mut(trigger.target()).unwrap();
                        npc.state = NPCState::Idle;
                    }
                    _ => {}
                }
            }
        }

        DialogActionEvent::Close => {}
    }
}

fn update_free_npcs(
    time: Res<Time>,
    mut decision_makers: Query<
        (&mut DecisionMaker, &GlobalTransform, &mut EntityController),
        With<FreeNPC>,
    >,
) {
    for (mut decision_maker, global_transform, mut entity_controller) in &mut decision_makers {
        match decision_maker.state {
            NPCState::Idle => {
                if decision_maker.decision_timer.tick(time.delta()).finished() {
                    let x = (random::<f32>() - 0.5) * 100.0;
                    let y = (random::<f32>() - 0.5) * 100.0;
                    let pos = global_transform.translation();

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

            NPCState::Interacting => {
                entity_controller.state = EntityState::Idle;
            }
        }
    }
}
