use bevy::camera::visibility::*;
use bevy::color::palettes::css::*;
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

pub fn render_other_players(
    mut commands: Commands,
    players: Query<Entity, (Without<Controlled>, Added<PlayerId>)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if players.is_empty() {
        return;
    }

    let material = materials.add(StandardMaterial {
        base_color: GRAY.into(),
        ..default()
    });
    let model = meshes.add(Capsule3d::new(CHARACTER_WIDTH, CHARACTER_HEIGHT));

    for entity in players.iter() {
        commands
            .entity(entity)
            .insert((Mesh3d(model.clone()), MeshMaterial3d(material.clone())));
    }
}

pub fn render_cubes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    cubes: Query<(Entity, Option<Ref<Glowing>>, Option<Ref<Light>>), Added<CubeMarker>>,
    floors: Query<(Entity, Option<Ref<Glowing>>, Option<Ref<Light>>), Added<FloorMarker>>,
) {
    if !cubes.is_empty() {
        let cube = meshes.add(Cuboid::from_length(1.0));
        let mut material = StandardMaterial {
            base_color: GRAY.into(),
            ..default()
        };

        for (entity, glowing, light) in cubes.iter() {
            if let Some(glowing) = glowing {
                material.emissive = LinearRgba::from(glowing.color);
            }

            let material = materials.add(material.clone());
            commands
                .entity(entity)
                .insert((Mesh3d(cube.clone()), MeshMaterial3d(material.clone())));

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

    if !floors.is_empty() {
        let mut material = StandardMaterial {
            base_color: GRAY.into(),
            ..default()
        };
        let floor = meshes.add(Cuboid::new(1.0, 1.0, 1.0));

        for (entity, glowing, light) in floors.iter() {
            if glowing.is_some() {
                // todo: glowing color
                material.emissive = LinearRgba::rgb(100.0, 20.0, 200.0);
            }

            let material = materials.add(material.clone());
            commands
                .entity(entity)
                .insert((Mesh3d(floor.clone()), MeshMaterial3d(material.clone())));

            if light.is_some() {
                commands.entity(entity).insert((
                    PointLight {
                        radius: 10.0,
                        ..default()
                    },
                    NoFrustumCulling,
                ));
            }
        }
    }
}
