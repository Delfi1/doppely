//! Сервер управялет ИИ агентами, занимается их обучением, а так-же работает с подключениями игроков.
//! Для создания нейросети используется крейт [dfdx](https://docs.rs/dfdx/latest/dfdx/)
//! Для игровой логики используется движок [bevy](https://docs.rs/bevy/latest/bevy/)
//! Для клиент-серверного взаимодействия используется [lightyear](https://docs.rs/lightyear/latest/lightyear/)

mod protocol;
pub use protocol::*;

use bevy::prelude::*;
pub use core::net::{IpAddr, Ipv4Addr, SocketAddr};

pub const TIMESTEP_HZ: f64 = 64.0;
pub const SERVER_PORT: u16 = 5000;
pub const PLAYER_SPEED: f32 = 5.0;
pub const SHIFT_MULTIPLIER: f32 = 1.7;

#[derive(Clone)]
pub struct SharedPlugin;

impl Plugin for SharedPlugin {
    fn build(&self, app: &mut App) {
        // todo: add physics avian3d

        app.add_plugins(ProtocolPlugin);
    }
}
