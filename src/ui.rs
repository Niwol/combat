use bevy::prelude::*;

use crate::{living_entity::player::Player, ui::ui_dialog::UiDialogPlugin, xp::XpInventory};

pub mod ui_dialog;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(UiDialogPlugin);

        app.add_observer(spawn_xp_text);

        app.add_systems(Update, update_xp_text);
    }
}

#[derive(Component)]
struct XpText;

fn spawn_xp_text(_trigger: Trigger<OnAdd, XpInventory>, mut commands: Commands) {
    commands.spawn((Text::new("Player Xp: 0"), XpText));
}

fn update_xp_text(
    mut xp_text: Single<&mut Text, With<XpText>>,
    player_xp: Single<&XpInventory, (Changed<XpInventory>, With<Player>)>,
) {
    xp_text.0 = format!("Player Xp: {}", player_xp.amount());
}
