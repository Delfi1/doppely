use bevy::color::palettes::css::*;
use bevy::prelude::*;
use doppely::*;
use lightyear::prelude::Controlled;

#[derive(Component)]
pub struct DebugText;

pub fn display_debug(
    mut commands: Commands,
    player: Single<Entity, With<Controlled>>,
    text: Option<Single<Mut<Text>, With<DebugText>>>,
) {
    let Some(mut text) = text else {
        commands.spawn((Text::new("Debug data: \n"), DebugText));
        return;
    };

    let data = format!("Player entity: {:?}", player.index_u32());
    bevy::log::debug!("{}", data);
    *text.as_deref_mut() = data;
}

pub fn _setup_players_model(
    mut commands: Commands,
    players: Query<Entity, (With<PlayerId>, Without<Mesh3d>)>,
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
    let sphere = meshes.add(Sphere::default());

    for entity in players.iter() {
        commands
            .entity(entity)
            .insert((Mesh3d(sphere.clone()), MeshMaterial3d(material.clone())));
    }
}
