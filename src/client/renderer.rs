use bevy::camera::visibility::*;
pub use bevy::diagnostic::*;
use bevy::prelude::*;
use doppely::*;
use lightyear::prelude::Controlled;

#[derive(Component)]
pub struct DebugText;

pub fn display_debug(
    mut commands: Commands,
    diagnostics: Res<DiagnosticsStore>,
    player: Option<Single<(Entity, Ref<Transform>), With<Controlled>>>,
    text: Option<Single<Mut<Text>, With<DebugText>>>,
) {
    let Some(mut text) = text else {
        commands.spawn((Text::new("_"), DebugText));
        return;
    };

    let Some(player) = player else {
        return;
    };

    let (entity, transform) = player.into_inner();

    let fps = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|fps| fps.smoothed())
        .unwrap_or(0.0)
        .round() as u64;

    let data = format!(
        "Fps: {}\nPlayer entity: {:?}\nCoords: {:?}",
        fps,
        entity,
        transform.translation.to_array()
    );
    bevy::log::debug!("{}", data);
    *text.as_deref_mut() = data;
}

pub fn render_objects(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    objects: Query<
        (
            Entity,
            Ref<ObjectMarker>,
            Option<Ref<Glowing>>,
            Option<Ref<Light>>,
        ),
        (Added<ObjectMarker>, Without<Controlled>),
    >,
) {
    for (entity, marker, glowing, light) in objects.iter() {
        let mesh = meshes.add(marker.mesh());
        let mut material = marker.material();

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
}
