use bevy::camera::Camera;
use bevy::input::mouse::MouseMotion;
use bevy::input_focus::InputFocus;
use bevy::picking::hover::HoverMap;
use bevy::prelude::*;
use bevy::text::EditableText;
use bevy::ui::ComputedNode;
use bevy::window::{CursorGrabMode, CursorOptions};
use bevy_react::prelude::{ReactEvents, ReactNode};
use doppely::Position;
use doppely::scene::SceneObject;
use lightyear::prelude::MessageSender;

use super::renderer::MainCamera;
use super::states::ClientState;
use super::ui::{EditorPicker, SceneChanged, SceneObjectWire};
use crate::{EditorAction, EditorChannel, EditorCommand};

const FLY_SPEED: f32 = 7.0;
const LOOK_SENSITIVITY: f32 = 0.0025;
const PITCH_LIMIT: f32 = std::f32::consts::FRAC_PI_2 - 0.02;
const DRAG_INTERVAL: f32 = 0.08;
const PUBLISH_EVERY: u32 = 14;

#[derive(Resource)]
pub struct EditorCamera {
    pub yaw: f32,
    pub pitch: f32,
    pub position: Vec3,
}

impl Default for EditorCamera {
    fn default() -> Self {
        Self {
            yaw: 0.0,
            pitch: -0.35,
            position: Vec3::new(0.0, 4.0, 10.0),
        }
    }
}

impl EditorCamera {
    pub fn transform(&self) -> Transform {
        Transform::from_translation(self.position).with_rotation(Quat::from_euler(
            EulerRot::YXZ,
            self.yaw,
            self.pitch,
            0.0,
        ))
    }
}

#[derive(Resource, Default)]
pub struct EditorSelection(pub Vec<u32>);

impl EditorSelection {
    pub fn has(&self, id: u32) -> bool {
        self.0.contains(&id)
    }

    fn select(&mut self, id: u32) {
        self.0.clear();
        self.0.push(id);
    }

    fn make_primary(&mut self, id: u32) {
        if let Some(position) = self.0.iter().position(|current| *current == id) {
            self.0.remove(position);
        }
        self.0.push(id);
    }

    fn toggle(&mut self, id: u32) {
        if let Some(position) = self.0.iter().position(|current| *current == id) {
            self.0.remove(position);
        } else {
            self.0.push(id);
        }
    }

    pub fn remove(&mut self, id: u32) {
        self.0.retain(|current| *current != id);
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }

    pub fn select_from_ui(&mut self, id: u32, additive: bool) {
        if additive {
            self.toggle(id);
        } else {
            self.select(id);
        }
    }
}

#[derive(Default)]
struct Drag {
    start: Vec2,
    from: Vec3,
    originals: Vec<(u32, Vec3)>,
}

#[derive(Resource, Default)]
struct Dragging(Option<Drag>);

#[derive(Resource, Default)]
struct HighlightMaterial(Option<Handle<StandardMaterial>>);

#[derive(Resource, Default)]
struct Highlighted(Vec<(Entity, Handle<StandardMaterial>)>);

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<EditorCamera>()
        .init_resource::<EditorSelection>()
        .init_resource::<Dragging>()
        .init_resource::<HighlightMaterial>()
        .init_resource::<Highlighted>()
        .add_systems(
            Update,
            (
                fly_camera,
                editor_pointer,
                highlight_selection,
                publish_scene,
            )
                .run_if(in_state(ClientState::Edit)),
        );
}

fn fly_camera(
    mut camera: ResMut<EditorCamera>,
    keys: Res<ButtonInput<KeyCode>>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut mouse: MessageReader<MouseMotion>,
    mut cursor: Query<&mut CursorOptions>,
    time: Res<Time>,
) {
    let mut delta = Vec2::ZERO;
    for motion in mouse.read() {
        delta += motion.delta;
    }

    if buttons.just_pressed(MouseButton::Right) {
        if let Ok(mut cursor) = cursor.single_mut() {
            cursor.grab_mode = CursorGrabMode::Locked;
            cursor.visible = false;
        }
    }

    if buttons.just_released(MouseButton::Right) {
        if let Ok(mut cursor) = cursor.single_mut() {
            cursor.grab_mode = CursorGrabMode::None;
            cursor.visible = true;
        }
    }

    if buttons.pressed(MouseButton::Right) {
        camera.yaw -= delta.x * LOOK_SENSITIVITY;
        camera.pitch = (camera.pitch - delta.y * LOOK_SENSITIVITY).clamp(-PITCH_LIMIT, PITCH_LIMIT);
    }

    let yaw = Quat::from_rotation_y(camera.yaw);
    let forward = yaw * Vec3::NEG_Z;
    let right = yaw * Vec3::X;

    let mut direction = Vec3::ZERO;
    if keys.pressed(KeyCode::KeyW) {
        direction += forward;
    }
    if keys.pressed(KeyCode::KeyS) {
        direction -= forward;
    }
    if keys.pressed(KeyCode::KeyD) {
        direction += right;
    }
    if keys.pressed(KeyCode::KeyA) {
        direction -= right;
    }
    if keys.pressed(KeyCode::Space) {
        direction += Vec3::Y;
    }
    if keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight) {
        direction -= Vec3::Y;
    }

    if direction != Vec3::ZERO {
        camera.position += direction.normalize() * FLY_SPEED * time.delta_secs();
    }
}

#[allow(clippy::too_many_arguments)]
fn editor_pointer(
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    scene: Query<(&SceneObject, &Position)>,
    keys: Res<ButtonInput<KeyCode>>,
    buttons: Res<ButtonInput<MouseButton>>,
    hover: Option<Res<HoverMap>>,
    ui: Query<(&ComputedNode, &ReactNode)>,
    focus: Option<Res<InputFocus>>,
    text_inputs: Query<(), With<EditableText>>,
    mut selection: ResMut<EditorSelection>,
    mut dragging: ResMut<Dragging>,
    mut senders: Query<&mut MessageSender<EditorCommand>>,
    events: ReactEvents,
    time: Res<Time>,
    mut last_move: Local<f32>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let Ok((camera, camera_transform)) = cameras.single() else {
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor) else {
        return;
    };
    let origin = ray.origin;
    let direction = ray.direction.as_vec3();
    let over_ui = cursor_over_ui(hover.as_deref(), &ui, window.size());
    let typing = focus
        .as_ref()
        .and_then(|focus| focus.get())
        .is_some_and(|entity| text_inputs.contains(entity));

    if keys.just_pressed(KeyCode::Delete) && !typing {
        if !selection.0.is_empty() {
            send(
                &mut senders,
                EditorAction::RemoveMany {
                    ids: selection.0.clone(),
                },
            );
        }
        selection.clear();
        dragging.0 = None;
    }

    let control = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    let shift = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);

    if control && !typing {
        let redo = keys.just_pressed(KeyCode::KeyY) || (shift && keys.just_pressed(KeyCode::KeyZ));
        if redo {
            send(&mut senders, EditorAction::Redo);
            dragging.0 = None;
        } else if keys.just_pressed(KeyCode::KeyZ) {
            send(&mut senders, EditorAction::Undo);
            dragging.0 = None;
        }
    }

    if keys.just_pressed(KeyCode::KeyF) && !over_ui && !typing {
        if let Some(point) = plane_hit(origin, direction) {
            events.send(&EditorPicker {
                x: cursor.x,
                y: cursor.y,
                position: point.to_array(),
            });
        }
    }

    if buttons.just_pressed(MouseButton::Left) && !over_ui {
        match pick(&scene, origin, direction) {
            Some(id) => {
                if shift || control {
                    selection.toggle(id);
                } else if selection.has(id) {
                    selection.make_primary(id);
                } else {
                    selection.select(id);
                }

                let originals = selection
                    .0
                    .iter()
                    .filter_map(|id| {
                        scene
                            .iter()
                            .find(|(object, _)| object.id == *id)
                            .map(|(object, position)| (object.id, position.0))
                    })
                    .collect();

                dragging.0 = plane_hit(origin, direction).map(|point| Drag {
                    start: cursor,
                    from: point,
                    originals,
                });
            }
            None => {
                if !shift && !control {
                    selection.clear();
                }
                dragging.0 = None;
            }
        }
    }

    if buttons.pressed(MouseButton::Left) && !over_ui {
        if let Some(drag) = dragging.0.as_ref() {
            let moved = drag.start.distance(cursor) > 3.0;
            if moved && time.elapsed_secs() - *last_move >= DRAG_INTERVAL {
                if let Some(point) = plane_hit(origin, direction) {
                    let delta = point - drag.from;
                    for (id, position) in &drag.originals {
                        send(
                            &mut senders,
                            EditorAction::Move {
                                id: *id,
                                position: Vec3::new(
                                    position.x + delta.x,
                                    position.y,
                                    position.z + delta.z,
                                ),
                            },
                        );
                    }
                    *last_move = time.elapsed_secs();
                }
            }
        }
    }

    if buttons.just_released(MouseButton::Left) {
        dragging.0 = None;
    }
}

pub fn cursor_over_ui(
    hover: Option<&HoverMap>,
    ui: &Query<(&ComputedNode, &ReactNode)>,
    window_size: Vec2,
) -> bool {
    let Some(hover) = hover else {
        return false;
    };
    hover.values().any(|hits| {
        hits.keys().any(|entity| {
            ui.get(*entity).is_ok_and(|(node, _)| {
                let size = node.size();
                size.x < window_size.x - 2.0 || size.y < window_size.y - 2.0
            })
        })
    })
}

fn send(senders: &mut Query<&mut MessageSender<EditorCommand>>, action: EditorAction) {
    for mut sender in senders.iter_mut() {
        sender.send::<EditorChannel>(EditorCommand {
            action: action.clone(),
        });
    }
}

fn highlight_selection(
    selection: Res<EditorSelection>,
    mut scene: Query<(Entity, &SceneObject, &mut MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut material: ResMut<HighlightMaterial>,
    mut highlighted: ResMut<Highlighted>,
) {
    let mut desired: Vec<Entity> = Vec::new();
    for id in &selection.0 {
        if let Some((entity, _, _)) = scene.iter().find(|(_, object, _)| object.id == *id) {
            desired.push(entity);
        }
    }

    let same = highlighted.0.len() == desired.len()
        && desired
            .iter()
            .all(|entity| highlighted.0.iter().any(|(current, _)| current == entity));
    if same {
        return;
    }

    for (entity, old) in highlighted.0.drain(..) {
        if let Ok((_, _, mut mesh)) = scene.get_mut(entity) {
            mesh.0 = old;
        }
    }

    if desired.is_empty() {
        return;
    }

    let handle = material
        .0
        .get_or_insert_with(|| {
            materials.add(StandardMaterial {
                base_color: Srgba::new(0.42, 0.62, 1.0, 1.0).into(),
                emissive: LinearRgba::from(Srgba::new(0.08, 0.18, 0.45, 1.0)),
                ..default()
            })
        })
        .clone();

    for entity in desired {
        if let Ok((_, _, mut mesh)) = scene.get_mut(entity) {
            let old = mesh.0.clone();
            mesh.0 = handle.clone();
            highlighted.0.push((entity, old));
        }
    }
}

fn publish_scene(
    scene: Query<&SceneObject>,
    mut selection: ResMut<EditorSelection>,
    events: ReactEvents,
    mut tick: Local<u32>,
) {
    *tick = tick.wrapping_add(1);
    if !tick.is_multiple_of(PUBLISH_EVERY) && !selection.is_changed() {
        return;
    }

    let mut objects: Vec<SceneObjectWire> = scene
        .iter()
        .map(|object| SceneObjectWire {
            id: object.id,
            name: object.name.clone(),
            kind: match object.kind {
                doppely::ObjectType::Cuboid => "cuboid",
                doppely::ObjectType::Sphere => "sphere",
                doppely::ObjectType::Capsule => "capsule",
                doppely::ObjectType::Cylinder => "cylinder",
                doppely::ObjectType::Cone => "cone",
                doppely::ObjectType::Spawnpoint => "spawnpoint",
            }
            .to_string(),
            position: object.position.to_array(),
            rotation: object.rotation.to_array(),
            size: object.size.to_array(),
            components: object.components.clone(),
        })
        .collect();
    objects.sort_by_key(|object| object.id);

    let total = objects.len();
    objects.dedup_by_key(|object| object.id);
    if objects.len() != total {
        bevy::log::warn!(
            "Сцена: одинаковые id у {} объект(ов), лишние строки скрыты",
            total - objects.len()
        );
    }

    let missing = selection
        .0
        .iter()
        .any(|id| !objects.iter().any(|object| object.id == *id));
    if missing {
        selection
            .0
            .retain(|id| objects.iter().any(|object| object.id == *id));
    }

    events.send(&SceneChanged {
        objects,
        selected: selection.0.clone(),
    });
}

fn plane_hit(origin: Vec3, direction: Vec3) -> Option<Vec3> {
    if direction.y.abs() < 1e-5 {
        return None;
    }
    let distance = -origin.y / direction.y;
    (distance > 0.0).then(|| origin + direction * distance)
}

fn pick(scene: &Query<(&SceneObject, &Position)>, origin: Vec3, direction: Vec3) -> Option<u32> {
    let mut best: Option<(f32, u32)> = None;
    for (object, position) in scene.iter() {
        let half = object.size * 0.5;
        if let Some(distance) = ray_box(origin, direction, position.0, half) {
            if best.is_none_or(|(nearest, _)| distance < nearest) {
                best = Some((distance, object.id));
            }
        }
    }
    best.map(|(_, id)| id)
}

fn ray_box(origin: Vec3, direction: Vec3, center: Vec3, half: Vec3) -> Option<f32> {
    let local = origin - center;
    let mut near = f32::NEG_INFINITY;
    let mut far = f32::INFINITY;

    for axis in 0..3 {
        let dir = direction[axis];
        let offset = local[axis];
        let extent = half[axis].max(0.01);

        if dir.abs() < 1e-6 {
            if offset.abs() > extent {
                return None;
            }
        } else {
            let first = (-extent - offset) / dir;
            let second = (extent - offset) / dir;
            near = near.max(first.min(second));
            far = far.min(first.max(second));
        }
    }

    (far >= near.max(0.0)).then_some(near.max(0.0))
}
