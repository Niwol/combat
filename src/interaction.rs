use avian2d::prelude::{
    Collider, CollisionEventsEnabled, OnCollisionEnd, OnCollisionStart, Sensor,
};
use bevy::prelude::*;

pub struct InteractionPlugin;

impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<TryInteractingEvent>();
        app.add_event::<InteractionEvent>();

        app.add_observer(observe_interaction_trys);
        app.add_observer(interactable_added);
    }
}

#[derive(Event)]
pub struct InteractionEvent {
    pub interactor: Entity,
}

#[derive(Event)]
pub struct TryInteractingEvent;

#[derive(Component)]
#[require(Collider, CollisionEventsEnabled, Sensor)]
pub struct Interactable {
    interactors: Vec<Entity>,
}

impl Interactable {
    pub fn new() -> Self {
        Self {
            interactors: Vec::new(),
        }
    }

    pub fn interactors(&self) -> Vec<Entity> {
        self.interactors.clone()
    }
}

impl Default for Interactable {
    fn default() -> Self {
        Self {
            interactors: Vec::default(),
        }
    }
}

#[derive(Component)]
pub struct Interactor;

fn observe_interaction_trys(
    trigger: Trigger<TryInteractingEvent>,
    mut commands: Commands,
    interactables: Query<(Entity, &Interactable)>,
) {
    let interactor = trigger.target();

    for (entity, interactable) in &interactables {
        if interactable.interactors.contains(&interactor) {
            commands.trigger_targets(InteractionEvent { interactor }, entity);
            break;
        }
    }
}

fn interactable_added(trigger: Trigger<OnAdd, Interactable>, mut commands: Commands) {
    commands
        .entity(trigger.target())
        .observe(start_collision)
        .observe(end_collision);
}

fn start_collision(
    trigger: Trigger<OnCollisionStart>,
    mut interactables: Query<&mut Interactable>,
    interactors: Query<Entity, With<Interactor>>,
) {
    let mut interactable = interactables.get_mut(trigger.target()).unwrap();
    if let Ok(interactor) = interactors.get(trigger.collider) {
        interactable.interactors.push(interactor);
    }
}

fn end_collision(
    trigger: Trigger<OnCollisionEnd>,
    mut interactables: Query<&mut Interactable>,
    interactors: Query<Entity, With<Interactor>>,
) {
    let mut interactable = interactables.get_mut(trigger.target()).unwrap();
    if let Ok(interactor) = interactors.get(trigger.collider) {
        if let Some(index) = interactable
            .interactors
            .iter()
            .position(|inter| *inter == interactor)
        {
            interactable.interactors.remove(index);
        }
    }
}
