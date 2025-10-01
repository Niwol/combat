use avian2d::prelude::Collider;
use bevy::{color::palettes, prelude::*, sprite::Anchor};
use bevy_ecs_tiled::prelude::*;

use crate::{
    character_controller::PlayerInputState,
    head_quarter,
    interaction::{Interactable, InteractionEvent},
    living_entity::player::Player,
    skills::SkillTree,
    ui::ui_dialog::{
        DialogAction, DialogActionEvent, DialogButton, DialogButtons, DialogNode, DialogTree,
    },
    xp::XpInventory,
};

pub struct HeadQuarterPlugin;

impl Plugin for HeadQuarterPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, load_sprites);

        app.add_observer(spawn);

        app.add_systems(Update, update_head_quarter);
    }
}

#[derive(Resource)]
struct HeadQuarterSprites {
    tent_spot: Handle<Image>,
    tent: Handle<Image>,
}

fn load_sprites(mut commands: Commands, assets: Res<AssetServer>) {
    let sprites = HeadQuarterSprites {
        tent_spot: assets.load("tent_spot.png"),
        tent: assets.load("tent.png"),
    };

    commands.insert_resource(sprites);
}

enum HeadQuarterState {
    TentSpot,
    Tent,
}

#[derive(Component)]
pub struct HeadQuarter {
    state: HeadQuarterState,
}

#[derive(Component)]
struct HeadQuarterText;

fn spawn(
    trigger: Trigger<TiledEvent<ObjectCreated>>,
    mut commands: Commands,
    mut objects: Query<(Entity, &Name, &mut Sprite)>,
) {
    let object = objects.get_mut(trigger.target());

    let Ok((entity, name, mut sprite)) = object else {
        return;
    };

    if *name != Name::new("Tile(Head Quarter)") {
        return;
    }

    sprite.anchor = Anchor::Center;

    commands
        .entity(entity)
        .insert((
            HeadQuarter {
                state: HeadQuarterState::TentSpot,
            },
            Interactable::new(),
            Collider::rectangle(20.0, 20.0),
            children![(
                Text2d::new("Build Tent (1000 XP)"),
                Transform::from_xyz(0.0, 30.0, 0.0),
                HeadQuarterText,
                Visibility::Hidden,
            )],
        ))
        .observe(observe_player_interaction)
        .observe(observe_dialog_action);
}

fn update_head_quarter(
    head_quarter: Single<(&Interactable, &HeadQuarter), With<HeadQuarter>>,
    mut head_quarter_text: Query<&mut Visibility, With<HeadQuarterText>>,
    player: Single<&Transform, With<Player>>,
) {
    let Ok(mut head_quarter_text) = head_quarter_text.single_mut() else {
        return;
    };

    if let HeadQuarterState::Tent = head_quarter.1.state {
        *head_quarter_text = Visibility::Hidden;
        return;
    }

    if !head_quarter.0.interactors().is_empty() {
        *head_quarter_text = Visibility::Inherited;
    } else {
        *head_quarter_text = Visibility::Hidden;
    }
}

fn observe_player_interaction(
    _trigger: Trigger<InteractionEvent>,
    mut commands: Commands,
    mut player: Single<(&mut XpInventory, &SkillTree), With<Player>>,
    mut head_quarter: Single<(Entity, &mut HeadQuarter, &mut Sprite)>,
    mut next_player_state: ResMut<NextState<PlayerInputState>>,
) {
    match head_quarter.1.state {
        HeadQuarterState::TentSpot => {
            if player.0.spend(1000) {
                head_quarter.2.rect = Some(Rect::new(32.0, 0.0, 64.0, 32.0));
                head_quarter.1.state = HeadQuarterState::Tent;

                // commands
                //     .entity(head_quarter.0)
                //     .remove_children(&[*head_quarter_text]);
                // commands.entity(*head_quarter_text).despawn();
            }
        }

        HeadQuarterState::Tent => {
            let dialog_tree = DialogTree::new(DialogNode {
                text: String::from("Learn skills"),
                action: DialogAction::Buttons(DialogButtons {
                    selected_button: 0,
                    buttons: {
                        let mut buttons = ["Bullets", "Fire ball", "Beam"]
                            .into_iter()
                            .filter_map(|skill_name| {
                                if !player.1.unlocked(skill_name.to_string()) {
                                    return Some(DialogButton::new(format!(
                                        "{skill_name} (1000Xp)"
                                    )));
                                }

                                None
                            })
                            .collect::<Vec<_>>();

                        buttons.push(DialogButton::new("Cancel"));

                        buttons
                    },
                }),
            });

            commands.entity(head_quarter.0).insert(dialog_tree);

            next_player_state.set(PlayerInputState::Menu);
        }
    }
}

fn observe_dialog_action(
    trigger: Trigger<DialogActionEvent>,
    mut commands: Commands,
    dialog_trees: Query<(Entity, &DialogTree)>,
    mut player: Single<(&mut XpInventory, &mut SkillTree)>,

    mut next_player_state: ResMut<NextState<PlayerInputState>>,
) {
    let (entity, dialog_tree) = dialog_trees.get(trigger.target()).unwrap();

    let node = &dialog_tree.nodes[dialog_tree.current_node];

    match &node.action {
        DialogAction::Buttons(dialog_buttons) => {
            let button = &dialog_buttons.buttons[dialog_buttons.selected_button];

            if let DialogActionEvent::Confirm = trigger.event() {
                match button.text.as_str() {
                    "Bullets (1000Xp)" => {
                        if player.0.spend(1000) {
                            player.1.unlock_skill("Bullets".into());
                        }
                    }

                    "Fire ball (1000Xp)" => {
                        if player.0.spend(1000) {
                            player.1.unlock_skill("Fire ball".into());
                        }
                    }

                    "Beam (1000Xp)" => {
                        if player.0.spend(1000) {
                            player.1.unlock_skill("Beam".into());
                        }
                    }

                    "Cancel" => {
                        next_player_state.set(PlayerInputState::CharacterController);
                        commands.entity(entity).remove::<DialogTree>();
                    }

                    _ => {}
                }
            }
        }

        DialogAction::NextNode(_) => {}
        DialogAction::End => {}
    }
}
