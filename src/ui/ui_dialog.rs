use bevy::{
    asset::RenderAssetUsages,
    color::palettes,
    prelude::*,
    render::{
        camera::RenderTarget,
        render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages},
        view::RenderLayers,
    },
    sprite::Anchor,
};

pub struct UiDialogPlugin;

impl Plugin for UiDialogPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<DialogActionEvent>();

        app.init_resource::<UiNavigator>();

        app.add_observer(dialog_tree_added);
        app.add_observer(dialog_tree_removed);

        app.add_systems(Update, (update_dialog_tree, update_dialog_ui).chain());
    }
}

#[derive(Resource)]
pub struct UiNavigator {
    confirm: bool,
    back: bool,

    ui_up: bool,
    ui_down: bool,
    ui_left: bool,
    ui_right: bool,
}

impl Default for UiNavigator {
    fn default() -> Self {
        Self {
            confirm: false,
            back: false,

            ui_up: false,
            ui_down: false,
            ui_left: false,
            ui_right: false,
        }
    }
}

impl UiNavigator {
    pub fn confirm(&mut self) {
        self.confirm = true;
    }
    pub fn back(&mut self) {
        self.back = true;
    }
    pub fn ui_up(&mut self) {
        self.ui_up = true;
    }
    pub fn ui_down(&mut self) {
        self.ui_down = true;
    }
    pub fn ui_left(&mut self) {
        self.ui_left = true;
    }
    pub fn ui_right(&mut self) {
        self.ui_right = true;
    }
}

#[derive(Event)]
pub enum DialogActionEvent {
    Confirm,
    Close,
}

#[derive(Component)]
struct UiEntities {
    ui_camera: Entity,
    ui_entity: Entity,
    ui_image_entity: Entity,
    ui_image: Handle<Image>,
}

#[derive(Component)]
pub struct DialogTree {
    pub nodes: Vec<DialogNode>,
    pub current_node: usize,
}

impl DialogTree {
    pub fn new(first_node: DialogNode) -> Self {
        Self {
            nodes: vec![first_node],
            current_node: 0,
        }
    }
}

pub struct DialogNode {
    pub text: String,
    pub action: DialogAction,
}

pub enum DialogAction {
    Buttons(DialogButtons),
    NextNode(usize),
    End,
}

pub struct DialogButtons {
    pub selected_button: usize,
    pub buttons: Vec<DialogButton>,
}

pub struct DialogButton {
    pub text: String,
    entity: Entity,
}

impl DialogButton {
    pub fn new(text: impl ToString) -> Self {
        Self {
            text: text.to_string(),
            entity: Entity::PLACEHOLDER,
        }
    }
}

pub enum ButtonAction {
    NextNode(usize),
}

fn dialog_tree_added(
    trigger: Trigger<OnAdd, DialogTree>,
    mut commands: Commands,
    mut dialog_trees: Query<&mut DialogTree>,
    mut images: ResMut<Assets<Image>>,
    names: Query<NameOrEntity>,
) {
    let mut image = Image::new_fill(
        Extent3d {
            width: 512,
            height: 256,
            ..Default::default()
        },
        TextureDimension::D2,
        &[0, 0, 0, 0],
        TextureFormat::Bgra8UnormSrgb,
        RenderAssetUsages::default(),
    );

    image.texture_descriptor.usage =
        TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST | TextureUsages::RENDER_ATTACHMENT;

    let handle = images.add(image);

    let target_camera = commands
        .spawn((
            Camera2d,
            Camera {
                target: RenderTarget::Image(handle.clone().into()),
                clear_color: Color::srgba(0.0, 0.0, 0.0, 0.0).into(),
                ..Default::default()
            },
            RenderLayers::layer(1),
        ))
        .id();

    let ui_image = commands
        .spawn((
            Sprite {
                image: handle.clone(),
                anchor: Anchor::BottomCenter,
                // custom_size: Some(Vec2::splat(64.0)),
                ..Default::default()
            },
            Transform::from_xyz(0.0, 16.0, 0.0).with_scale(Vec3::splat(0.25)),
        ))
        .id();

    let name = names.get(trigger.target()).unwrap();
    let root = commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                border: UiRect::all(Val::Px(5.0)),
                padding: UiRect::all(Val::Px(8.0)),

                justify_self: JustifySelf::Center,
                align_self: AlignSelf::End,
                ..Default::default()
            },
            BorderColor(palettes::basic::BLACK.into()),
            BorderRadius::all(Val::Px(10.0)),
            BackgroundColor(palettes::css::BROWN.into()),
            UiTargetCamera(target_camera),
            RenderLayers::layer(1),
            children![(
                Text::new(format!("{name}")),
                TextFont {
                    font_size: 32.0,
                    ..Default::default()
                },
            )],
        ))
        .id();

    let mut dialog_tree = dialog_trees.get_mut(trigger.target()).unwrap();

    let node = &mut dialog_tree.nodes[0];

    let node_root = commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                ..Default::default()
            },
            children![Text::new(&node.text),],
        ))
        .id();

    match &mut node.action {
        DialogAction::Buttons(dialog_buttons) => {
            for (i, dialog_button) in dialog_buttons.buttons.iter_mut().enumerate() {
                let button = commands
                    .spawn((
                        Node {
                            border: UiRect::all(Val::Px(2.0)),
                            ..Default::default()
                        },
                        BorderColor(
                            palettes::basic::BLACK
                                .with_alpha(if i == dialog_buttons.selected_button {
                                    1.0
                                } else {
                                    0.0
                                })
                                .into(),
                        ),
                        children![(
                            Text::new(&dialog_button.text),
                            TextFont {
                                font_size: 16.0,
                                ..Default::default()
                            },
                        )],
                    ))
                    .id();

                dialog_button.entity = button;

                commands.entity(node_root).add_child(button);
            }
        }

        DialogAction::NextNode(_) => {}
        DialogAction::End => {}
    }

    commands.entity(root).add_child(node_root);

    commands
        .entity(trigger.target())
        .insert(UiEntities {
            ui_camera: target_camera,
            ui_entity: root,
            ui_image_entity: ui_image,
            ui_image: handle.clone(),
        })
        .add_child(ui_image);
}

fn update_dialog_tree(
    mut commands: Commands,
    mut ui_navigator: ResMut<UiNavigator>,
    mut dialog_trees: Query<(Entity, &mut DialogTree)>,
) {
    for (entity, mut dialog_tree) in &mut dialog_trees {
        let current_index = dialog_tree.current_node;
        let current_node = &mut dialog_tree.nodes[current_index];

        let mut next_node = None;

        match &mut current_node.action {
            DialogAction::Buttons(dialog_buttons) => {
                if ui_navigator.ui_up {
                    if dialog_buttons.selected_button == 0 {
                        dialog_buttons.selected_button = dialog_buttons.buttons.len() - 1;
                    } else {
                        dialog_buttons.selected_button -= 1;
                    }
                }

                if ui_navigator.ui_down {
                    dialog_buttons.selected_button += 1;
                    if dialog_buttons.selected_button == dialog_buttons.buttons.len() {
                        dialog_buttons.selected_button = 0;
                    }
                }

                if ui_navigator.confirm {
                    commands.trigger_targets(DialogActionEvent::Confirm, entity);
                }
            }

            DialogAction::NextNode(index) => {
                if ui_navigator.confirm {
                    next_node = Some(index);
                }
            }

            DialogAction::End => {}
        }

        if let Some(index) = next_node {
            dialog_tree.current_node = *index;
        }
    }

    *ui_navigator = UiNavigator::default();
}

fn update_dialog_ui(dialog_trees: Query<&DialogTree>, mut border_colors: Query<&mut BorderColor>) {
    for dialog_tree in &dialog_trees {
        let current_node = &dialog_tree.nodes[dialog_tree.current_node];

        if let DialogAction::Buttons(buttons) = &current_node.action {
            for (i, button) in buttons.buttons.iter().enumerate() {
                let mut border_color = border_colors.get_mut(button.entity).unwrap();

                let alpha = if buttons.selected_button == i {
                    1.0
                } else {
                    0.0
                };

                border_color.0 = palettes::basic::BLACK.with_alpha(alpha).into();
            }
        }
    }
}

fn dialog_tree_removed(
    trigger: Trigger<OnRemove, DialogTree>,
    mut commands: Commands,
    ui_entities: Query<&UiEntities>,
    mut assets: ResMut<Assets<Image>>,
) {
    let ui_entities = ui_entities.get(trigger.target()).unwrap();

    commands.entity(ui_entities.ui_camera).despawn();
    commands.entity(ui_entities.ui_entity).despawn();
    commands.entity(ui_entities.ui_image_entity).despawn();

    assets.remove(&ui_entities.ui_image);
}
