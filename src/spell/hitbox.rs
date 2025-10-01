use avian2d::prelude::*;
use bevy::prelude::*;

pub struct HitboxPlugin;

impl Plugin for HitboxPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnHitboxEvent>();

        app.add_observer(spawn_hitbox);

        app.add_systems(Update, update_hitboxes);
    }
}

#[derive(Event, Clone)]
pub struct SpawnHitboxEvent(pub HitboxInfo);
#[derive(Event, Clone, Copy)]
pub struct HitEntityEvent(pub Entity);

#[derive(Clone)]
pub struct HitboxInfo {
    pub size: Vec2,
    pub parent: Entity,
    pub live_timer: Option<Timer>,
}

#[derive(Component)]
struct Hitbox {
    live_timer: Option<Timer>,
}

fn spawn_hitbox(trigger: Trigger<SpawnHitboxEvent>, mut commands: Commands) {
    let hitbox = commands
        .spawn((
            Collider::rectangle(trigger.0.size.x, trigger.0.size.y),
            Sensor,
            CollisionEventsEnabled,
            Hitbox {
                live_timer: trigger.0.live_timer.clone(),
            },
        ))
        .observe(check_collisions)
        .id();

    commands
        .get_entity(trigger.0.parent)
        .unwrap()
        .add_child(hitbox);
}

fn check_collisions(
    trigger: Trigger<OnCollisionStart>,
    mut commands: Commands,
    childs_of: Query<&ChildOf>,
) {
    let parent = childs_of.get(trigger.target()).unwrap().parent();
    commands.trigger_targets(HitEntityEvent(trigger.collider), parent);
}

fn update_hitboxes(
    mut commands: Commands,
    time: Res<Time>,
    mut hitboxes: Query<(Entity, &mut Hitbox)>,
) {
    let delta = time.delta();

    for (entity, mut hitbox) in &mut hitboxes {
        if let Some(live_timer) = &mut hitbox.live_timer {
            live_timer.tick(delta);

            if live_timer.finished() {
                commands.entity(entity).despawn();
            }
        }
    }
}
