use doppely::*;

use bevy::color::palettes::css::*;
use bevy::prelude::*;
use core::net::{IpAddr, Ipv4Addr, SocketAddr};
use lightyear::prelude::client::ClientPlugins;
use lightyear::prelude::client::*;
use lightyear::prelude::*;
use std::time::Duration;

pub struct ClientPlugin;

const CLIENT_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 4000);

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, startup);
        app.add_systems(PostUpdate, (setup_players, update_players).chain());
    }
}

fn startup(mut commands: Commands) {
    let mut client = commands.spawn((
        Client,
        LocalAddr(CLIENT_ADDR),
        PeerAddr(SERVER_ADDR),
        Link::default(),
        ReplicationReceiver,
        RawClient,
        UdpIo::default(),
    ));
    client.trigger(Connect::from);

    commands.spawn((
        Transform::from_xyz(-5.0, 0.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
        Camera3d::default(),
    ));

    commands.spawn(DirectionalLight::default());
}

pub fn setup_players(
    mut commands: Commands,
    players: Query<(Entity, Ref<PlayerPosition>), Without<Mesh3d>>,
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

    for (entity, position) in players.iter() {
        commands.entity(entity).insert((
            Mesh3d(sphere.clone()),
            position.transform(),
            MeshMaterial3d(material.clone()),
        ));
    }
}

pub fn update_players(players: Query<(Ref<PlayerPosition>, Mut<Transform>)>) {
    for (position, mut transform) in players {
        *transform = position.transform();

        bevy::log::info!("Drawing player at {:?}", position.0);
    }
}

fn main() {
    let mut app = App::new();

    let delta = Duration::from_secs_f64(1.0 / TIMESTEP_HZ);
    app.add_plugins(DefaultPlugins);
    app.add_plugins(ClientPlugins {
        tick_duration: delta,
    });

    app.add_plugins(SharedPlugin);
    app.add_plugins(ClientPlugin);

    app.run();
}
