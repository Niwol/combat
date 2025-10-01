use bevy::prelude::*;

use crate::{
    interaction::TryInteractingEvent,
    living_entity::{
        EntityController, EntityState,
        player::{Player, PlayerInteractor},
    },
    skills::SkillTree,
    spell::{
        basic_attack::CastBasicAttack, beam::CastBeamSpell, bullets::CastBulletsSpell,
        fire_ball::CastFireBallSpell,
    },
    ui::ui_dialog::UiNavigator,
};

pub struct CharacterControllerPlugin;

impl Plugin for CharacterControllerPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<PlayerInputState>();

        app.init_resource::<InputMap>();
        app.init_resource::<MenuInputMap>();

        app.add_systems(
            Update,
            (handle_entity_controller, handle_inputs)
                .run_if(in_state(PlayerInputState::CharacterController)),
        );

        app.add_systems(
            Update,
            handle_menu_navigation.run_if(in_state(PlayerInputState::Menu)),
        );
    }
}

#[derive(Resource)]
pub struct InputMap {
    pub move_up: KeyCode,
    pub move_down: KeyCode,
    pub move_left: KeyCode,
    pub move_right: KeyCode,

    pub interact: KeyCode,
    pub attack: KeyCode,
    pub spell_1: KeyCode,
    pub spell_2: KeyCode,
    pub spell_3: KeyCode,
}

impl Default for InputMap {
    fn default() -> Self {
        Self {
            move_up: KeyCode::KeyW,
            move_down: KeyCode::KeyS,
            move_left: KeyCode::KeyA,
            move_right: KeyCode::KeyD,

            interact: KeyCode::Space,
            attack: KeyCode::KeyN,
            spell_1: KeyCode::KeyJ,
            spell_2: KeyCode::KeyK,
            spell_3: KeyCode::KeyL,
        }
    }
}

#[derive(Resource)]
pub struct MenuInputMap {
    pub ui_up: KeyCode,
    pub ui_down: KeyCode,
    pub ui_left: KeyCode,
    pub ui_right: KeyCode,

    pub ui_confirm: KeyCode,
    pub ui_back: KeyCode,
}

impl Default for MenuInputMap {
    fn default() -> Self {
        Self {
            ui_up: KeyCode::KeyW,
            ui_down: KeyCode::KeyS,
            ui_left: KeyCode::KeyA,
            ui_right: KeyCode::KeyD,

            ui_confirm: KeyCode::Space,
            ui_back: KeyCode::Escape,
        }
    }
}

#[derive(States, Default, Debug, PartialEq, Eq, Hash, Clone)]
pub enum PlayerInputState {
    #[default]
    CharacterController,
    Menu,
}

#[derive(Component)]
pub struct CharacterController {
    pub current_speed: f32,
    pub max_speed: f32,
}

fn handle_entity_controller(
    input: Res<ButtonInput<KeyCode>>,
    input_map: Res<InputMap>,
    mut entity_controller: Single<&mut EntityController, With<Player>>,
) {
    let mut direction = Vec2::ZERO;
    if input.pressed(input_map.move_up) {
        direction += Vec2::Y;
    }
    if input.pressed(input_map.move_down) {
        direction += Vec2::NEG_Y;
    }
    if input.pressed(input_map.move_left) {
        direction += Vec2::NEG_X;
    }
    if input.pressed(input_map.move_right) {
        direction += Vec2::X;
    }

    if direction != Vec2::ZERO {
        let direction = direction.normalize();
        entity_controller.state = EntityState::Move { direction };
    } else {
        entity_controller.state = EntityState::Idle;
    }
}

fn handle_inputs(
    input: Res<ButtonInput<KeyCode>>,
    input_map: Res<InputMap>,
    mut commands: Commands,
    player: Single<(Entity, &mut SkillTree), With<Player>>,
    player_interactor: Single<Entity, With<PlayerInteractor>>,
) {
    let (player, mut skill_tree) = player.into_inner();

    if input.just_pressed(input_map.attack) {
        let skill = skill_tree.skill_mut("Slash".into());
        if skill.unlocked && skill.reload_timer.finished() {
            commands.trigger_targets(CastBasicAttack, player);
            skill.reload_timer.reset();
        }
    }

    if input.just_pressed(input_map.spell_1) {
        let skill = skill_tree.skill_mut("Bullets".into());
        if skill.unlocked && skill.reload_timer.finished() {
            commands.trigger_targets(CastBulletsSpell, player);
            skill.reload_timer.reset();
        }
    }

    if input.just_pressed(input_map.spell_2) {
        let skill = skill_tree.skill_mut("Fire ball".into());

        if skill.unlocked && skill.reload_timer.finished() {
            commands.trigger_targets(CastFireBallSpell, player);
            skill.reload_timer.reset();
        }
    }

    if input.just_pressed(input_map.spell_3) {
        let skill = skill_tree.skill_mut("Beam".into());

        if skill.unlocked && skill.reload_timer.finished() {
            commands.trigger_targets(CastBeamSpell, player);
            skill.reload_timer.reset();
        }
    }

    if input.just_pressed(input_map.interact) {
        commands.trigger_targets(TryInteractingEvent, *player_interactor);
    }
}

fn handle_menu_navigation(
    input: Res<ButtonInput<KeyCode>>,
    input_map: Res<MenuInputMap>,
    mut ui_navigator: ResMut<UiNavigator>,
) {
    if input.just_pressed(input_map.ui_back) {
        ui_navigator.back();
    }

    if input.just_pressed(input_map.ui_confirm) {
        ui_navigator.confirm();
    }

    if input.just_pressed(input_map.ui_up) {
        ui_navigator.ui_up();
    }

    if input.just_pressed(input_map.ui_down) {
        ui_navigator.ui_down();
    }

    if input.just_pressed(input_map.ui_left) {
        ui_navigator.ui_left();
    }

    if input.just_pressed(input_map.ui_right) {
        ui_navigator.ui_right();
    }
}
