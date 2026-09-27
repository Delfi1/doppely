//! Сервер управялет ИИ агентами, занимается их обучением, а так-же работает с подключениями игроков.
//! Для создания нейросети используется крейт [dfdx](https://docs.rs/dfdx/latest/dfdx/)
//! Для игровой логики используется движок [bevy](https://docs.rs/bevy/latest/bevy/)
//! Для клиент-серверного взаимодействия используется [lightyear](https://docs.rs/lightyear/latest/lightyear/)

use bevy::ecs::entity::MapEntities;
use bevy::prelude::*;
use core::net::{IpAddr, Ipv4Addr, SocketAddr};
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

impl PlayerPosition {
    pub fn transform(&self) -> Transform {
        Transform::default().with_translation(self.0)
    }
}

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
