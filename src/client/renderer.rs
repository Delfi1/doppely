use bevy::{camera::visibility::NoFrustumCulling, post_process::bloom::Bloom, prelude::*};
use doppely::*;
use lightyear::prediction::Predicted;
use lightyear::prelude::Controlled;
use lightyear_frame_interpolation::{FrameInterpolate, FrameInterpolationPlugin};

pub struct RenderingPlugin;

impl Plugin for RenderingPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(handle_objects);
        app.add_observer(add_interpolation);
        app.add_observer(camera_observer);

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

    let mesh = meshes.add(object.mesh(size.0));
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

// Current camera marker
#[derive(Component, Clone, Copy)]
pub struct CurrentCamera;

pub fn ui_camera() -> impl Bundle {
    (Camera2d::default(), CurrentCamera)
}

pub fn controller_camera() -> impl Bundle {
    (Camera3d::default(), Bloom::default(), CurrentCamera)
}

fn camera_observer(
    trigger: On<Add<CurrentCamera>>,
    mut commands: Commands,
    other: Query<Entity, With<CurrentCamera>>,
) {
    for e in other {
        if e == trigger.entity {
            continue;
        }

        commands.entity(e).try_despawn();
    }
}
