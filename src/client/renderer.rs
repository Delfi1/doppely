use bevy::{camera::visibility::NoFrustumCulling, post_process::bloom::Bloom, prelude::*};
use doppely::*;
use lightyear::prediction::Predicted;
use lightyear::prelude::Controlled;
use lightyear_frame_interpolation::{FrameInterpolate, FrameInterpolationPlugin};

use super::editor::EditorCamera;
use super::game::CameraLook;
use super::states::ClientState;

pub struct RenderingPlugin;

impl Plugin for RenderingPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(handle_objects);
        app.add_observer(add_interpolation);
        app.add_systems(Update, (resize_objects, spawnpoint_gizmo));
        app.add_systems(Startup, spawn_camera);
        app.add_systems(
            PostUpdate,
            sync_camera
                .after(PhysicsSystems::Writeback)
                .before(TransformSystems::Propagate),
        );

        if !app.is_plugin_added::<FrameInterpolationPlugin>() {
            app.add_plugins(FrameInterpolationPlugin);
        }
    }
}

pub fn handle_objects(
    trigger: On<Add<ObjectType>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    object: Query<
        (
            Ref<ObjectType>,
            Ref<ObjectSize>,
            Option<Ref<Glowing>>,
            Option<Ref<Light>>,
        ),
        Without<Controlled>,
    >,
) {
    let entity = trigger.entity;
    let Ok((object, size, glowing, light)) = object.get(entity) else {
        return;
    };

    if *object == ObjectType::Spawnpoint {
        let mesh = meshes.add(Sphere::new(0.2));
        let material = materials.add(StandardMaterial {
            base_color: object.base_color().with_alpha(0.45).into(),
            emissive: LinearRgba::from(object.accent()),
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            ..default()
        });
        commands.entity(entity).insert((
            Mesh3d(mesh),
            MeshMaterial3d(material),
            SpawnpointGizmo,
            Visibility::Hidden,
        ));
        return;
    }

    let Some(mesh) = object.mesh(size.0) else {
        return;
    };
    let mesh = meshes.add(mesh);
    let mut material = object.material();

    if let Some(glowing) = glowing {
        material.emissive = LinearRgba::from(glowing.color);
    }

    let material = materials.add(material.clone());
    commands
        .entity(entity)
        .insert((Mesh3d(mesh.clone()), MeshMaterial3d(material.clone())));

    if let Some(light) = light {
        commands.entity(entity).insert((
            PointLight {
                radius: light.radius,
                intensity: light.intensity,
                ..default()
            },
            NoFrustumCulling,
        ));
    }
}

#[derive(Component)]
pub struct SpawnpointGizmo;

fn spawnpoint_gizmo(
    state: Res<State<ClientState>>,
    mut gizmos: Query<&mut Visibility, With<SpawnpointGizmo>>,
) {
    let visible = *state.get() == ClientState::Edit;
    for mut visibility in &mut gizmos {
        let desired = if visible {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if *visibility != desired {
            *visibility = desired;
        }
    }
}

fn resize_objects(
    objects: Query<(Entity, &ObjectType, Ref<ObjectSize>), Changed<ObjectSize>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
) {
    for (entity, object, size) in &objects {
        if size.is_added() {
            continue;
        }
        let Some(mesh) = object.mesh(size.0) else {
            continue;
        };
        commands.entity(entity).insert(Mesh3d(meshes.add(mesh)));
    }
}

fn add_interpolation(
    trigger: On<Add<Position>>,
    query: Query<&RigidBody, With<Predicted>>,
    clients: Query<(), With<lightyear::connection::client::Client>>,
    mut commands: Commands,
) {
    if clients.is_empty() {
        return;
    }
    let Ok(rb) = query.get(trigger.entity) else {
        return;
    };

    if rb.is_static() {
        return;
    }

    commands.entity(trigger.entity).insert(FrameInterpolate);
}

#[derive(Component)]
pub struct MainCamera;

pub fn main_camera() -> impl Bundle {
    (
        Name::new("Main camera"),
        Camera3d::default(),
        Bloom::default(),
        IsDefaultUiCamera,
        MainCamera,
        menu_camera_transform(),
    )
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(main_camera());
}

fn menu_camera_transform() -> Transform {
    Transform::from_xyz(0.0, 2.0, 8.0).looking_at(Vec3::new(0.0, 2.0, 0.0), Vec3::Y)
}

fn sync_camera(
    state: Res<State<ClientState>>,
    look: Res<CameraLook>,
    editor: Res<EditorCamera>,
    player: Query<&Transform, (With<Controlled>, Without<MainCamera>)>,
    mut camera: Query<&mut Transform, With<MainCamera>>,
) {
    let Ok(mut camera) = camera.single_mut() else {
        return;
    };

    match state.get() {
        ClientState::InGame | ClientState::Paused => {
            let Ok(player) = player.single() else {
                return;
            };

            camera.translation = player.translation;
            camera.rotation = player.rotation * Quat::from_rotation_x(look.pitch);
        }
        ClientState::Edit => *camera = editor.transform(),
        _ => *camera = menu_camera_transform(),
    }
}
