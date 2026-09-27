use crate::shared::*;
use bevy::math::VectorSpace;
use bevy::prelude::*;
use core::net::{IpAddr, Ipv4Addr, SocketAddr};
use lightyear::input::client::ClientInputPlugin;
use lightyear::prelude::client::*;
use lightyear::prelude::*;

pub struct ClientPlugin;

const CLIENT_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 4000);

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, startup);
        app.add_systems(PostUpdate, draw_players);
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
}

pub fn draw_players(mut gizmos: Gizmos, players: Query<&PlayerPosition>) {
    for position in &players {
        gizmos.sphere(Isometry3d::from_translation(position.0), 0.5, Color::WHITE);
        bevy::log::info!("Drawing player at {:?}", position.0);
    }
}
