use bevy::ecs::entity::MapEntities;
use bevy::prelude::*;
use core::net::{IpAddr, Ipv4Addr, SocketAddr};
use core::time::Duration;
use lightyear::input::native::prelude::*;
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};

pub const TIMESTEP_HZ: f64 = 64.0;
pub const SERVER_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 5000);

#[derive(Clone)]
pub struct SharedPlugin;

pub struct Channel1;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Message1(pub usize);

#[derive(
    Component, Clone, Debug, PartialEq, Reflect, Deref, DerefMut, Default, Serialize, Deserialize,
)]
pub struct PlayerPosition(pub Vec3);

#[derive(Serialize, Deserialize, Default, Debug, PartialEq, Eq, Clone, Copy, Hash, Reflect)]
pub struct MyInput {
    pub forward: bool,
    pub left: bool,
    pub right: bool,
    pub space: bool,
    pub catch: bool,
}

impl MapEntities for MyInput {
    fn map_entities<M: EntityMapper>(&mut self, _entity_mapper: &mut M) {}
}

impl Plugin for SharedPlugin {
    fn build(&self, app: &mut App) {
        app.register_message::<Message1>()
            .add_direction(NetworkDirection::Bidirectional);

        app.add_channel::<Channel1>(ChannelSettings {
            mode: ChannelMode::OrderedReliable(ReliableSettings::default()),
            ..default()
        })
        .add_direction(NetworkDirection::Bidirectional);

        app.component::<PlayerPosition>().replicate();

        app.add_plugins(InputPlugin::<MyInput>::default());
    }
}
