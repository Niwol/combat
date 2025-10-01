use bevy::{color::palettes, prelude::*};

pub struct XpPlugin;

impl Plugin for XpPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnXpEvent>();

        app.add_systems(Update, (start_collecting, update_xp, pick_up_xp));

        app.add_observer(spawn_xp);
    }
}

#[derive(Event)]
pub struct SpawnXpEvent {
    pub location: Vec2,
    pub amount: i32,
}

#[derive(Component)]
struct Xp(i32);

#[derive(Component)]
struct Collecting {
    collected_by: Entity,
    start_location: Transform,
    timer: Timer,
}

#[derive(Component)]
pub struct XpInventory {
    amount: i32,
}

impl XpInventory {
    pub fn new() -> Self {
        Self { amount: 0 }
    }

    pub fn collect_xp(&mut self, amount: i32) {
        self.amount += amount;
    }

    pub fn spend(&mut self, amount: i32) -> bool {
        if self.amount >= amount {
            self.amount -= amount;
            return true;
        }

        false
    }

    pub fn amount(&self) -> i32 {
        self.amount
    }
}

fn spawn_xp(trigger: Trigger<SpawnXpEvent>, mut commands: Commands) {
    let mut xp_to_spawn = trigger.amount;

    while xp_to_spawn != 0 {
        let max_xp = i32::min(xp_to_spawn, 5);
        let xp_amount = rand::random_range(1..=max_xp);

        xp_to_spawn -= xp_amount;
        let size = Vec2::splat(1.0 + xp_amount as f32);

        let offset = Vec2 {
            x: (rand::random::<f32>() - 0.5) * 5.0,
            y: (rand::random::<f32>() - 0.5) * 5.0,
        };

        commands.spawn((
            Sprite {
                color: palettes::basic::LIME.into(),
                custom_size: Some(size),
                ..Default::default()
            },
            Transform::from_xyz(
                trigger.location.x + offset.x,
                trigger.location.y + offset.y,
                0.0,
            ),
            Xp(xp_amount),
        ));
    }
}

fn start_collecting(
    mut commands: Commands,
    collectors: Query<(Entity, &Transform), With<XpInventory>>,
    xp: Query<(Entity, &Transform), (With<Xp>, Without<Collecting>)>,
) {
    for (collector, transform) in &collectors {
        for (xp, xp_transform) in &xp {
            let dist = transform.translation - xp_transform.translation;
            let dist = dist.length();

            if dist > 500.0 {
                continue;
            }

            commands.entity(xp).insert(Collecting {
                collected_by: collector,
                start_location: *xp_transform,
                timer: Timer::from_seconds(1.0, TimerMode::Once),
            });
        }
    }
}

fn update_xp(
    mut commands: Commands,
    time: Res<Time>,
    collectors: Query<&Transform, With<XpInventory>>,
    mut xp: Query<(Entity, &mut Transform, &mut Collecting), Without<XpInventory>>,
) {
    for (xp, mut xp_transform, mut collecting) in &mut xp {
        match collectors.get(collecting.collected_by) {
            Ok(transform) => {
                collecting.timer.tick(time.delta());

                let start_transform = collecting.start_location;
                let end_transform = transform;

                let progression = collecting.timer.elapsed().as_secs_f32();
                let easing = easing::EaseFunction::BackIn;
                let progression = easing.sample(progression).unwrap();

                let new_transform =
                    Transform::interpolate(&start_transform, end_transform, progression);

                *xp_transform = new_transform
            }

            Err(_) => {
                commands.entity(xp).remove::<Collecting>();
            }
        }
    }
}

fn pick_up_xp(
    mut commands: Commands,
    mut collector: Query<&mut XpInventory>,
    xp: Query<(Entity, &Xp, &Collecting)>,
) {
    for mut xp_inventory in &mut collector {
        for (entity, xp, collecting) in &xp {
            if collecting.timer.finished() {
                xp_inventory.amount += xp.0;
                commands.entity(entity).despawn();
            }
        }
    }
}
