use bevy::prelude::*;
use bullets::CastBulletsSpell;
use hitbox::HitboxPlugin;

pub mod basic_attack;
pub mod beam;
pub mod bullets;
pub mod fire_ball;
pub mod hitbox;

pub struct SpellPlugin;

impl Plugin for SpellPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(HitboxPlugin);

        app.add_plugins((
            basic_attack::plugin,
            bullets::plugin,
            fire_ball::plugin,
            beam::plugin,
        ));
    }
}
