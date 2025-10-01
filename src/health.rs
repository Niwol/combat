use bevy::prelude::*;

pub struct HealthPlugin;

impl Plugin for HealthPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, load_health_point);
        app.add_observer(add_health_bar);

        app.add_systems(PostUpdate, update_health_bar);
    }
}

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub struct Health(pub i32);

#[derive(Resource)]
struct HealtPointSprite {
    handle: Handle<Image>,
}

#[derive(Component)]
struct HealthBar;

fn load_health_point(mut commands: Commands, assetes: Res<AssetServer>) {
    let health_point_sprite = HealtPointSprite {
        handle: assetes.load("health_point.png"),
    };

    commands.insert_resource(health_point_sprite);
}

fn add_health_bar(
    trigger: Trigger<OnAdd, Health>,
    mut commands: Commands,
    query: Query<&Health>,
    health_point_sprite: Res<HealtPointSprite>,
) {
    let health = query.get(trigger.target()).unwrap();

    let health_point = commands
        .spawn((
            Sprite {
                custom_size: Some(Vec2 {
                    x: health.0 as f32 * 5.0 as f32,
                    y: 5.0,
                }),
                image: health_point_sprite.handle.clone(),
                image_mode: SpriteImageMode::Tiled {
                    tile_x: true,
                    tile_y: true,
                    stretch_value: 1.0,
                },
                ..Default::default()
            },
            Transform::from_xyz(0.0, 15.0, 0.0),
            HealthBar,
        ))
        .id();

    commands.entity(trigger.target()).add_child(health_point);
}

fn update_health_bar(
    mut entites: Query<(&Children, &Health), Changed<Health>>,
    mut healt_bars: Query<&mut Sprite, With<HealthBar>>,
) {
    for (children, health) in &mut entites {
        for child in children {
            if let Ok(mut sprite) = healt_bars.get_mut(*child) {
                sprite.custom_size = Some(Vec2 {
                    x: health.0 as f32 * 5.0,
                    y: 5.0,
                });
            }
        }
    }
}
