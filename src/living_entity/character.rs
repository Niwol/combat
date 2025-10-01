use bevy::prelude::*;

pub struct CharacterPlugin;

impl Plugin for CharacterPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, load_sprites);
    }
}

#[derive(Resource)]
pub struct CharacterSprites {
    pub generic: Handle<Image>,
    pub bard: Handle<Image>,
    pub soldier: Handle<Image>,
    pub scout: Handle<Image>,
    pub devout: Handle<Image>,
    pub conjurer: Handle<Image>,
}

pub fn load_sprites(mut commands: Commands, assets: Res<AssetServer>) {
    let character_sprites = CharacterSprites {
        generic: assets.load("characters/01-generic.png"),
        bard: assets.load("characters/02-bard.png"),
        soldier: assets.load("characters/03-soldier.png"),
        scout: assets.load("characters/04-scout.png"),
        devout: assets.load("characters/05-devout.png"),
        conjurer: assets.load("characters/06-conjurer.png"),
    };

    commands.insert_resource(character_sprites);
}
