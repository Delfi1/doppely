//! Сервер управялет ИИ агентами, занимается их обучением, а так-же работает с подключениями игроков.
//! Для создания нейросети используется крейт [dfdx](https://docs.rs/dfdx/latest/dfdx/)
//! Для игровой логики используется движок [bevy](https://docs.rs/bevy/latest/bevy/)
//! Для клиент-серверного взаимодействия используется [lightyear](https://docs.rs/lightyear/latest/lightyear/)

mod player;
mod protocol;
pub use player::*;
pub use protocol::*;

use bevy::prelude::*;
pub use core::net::*;
use lightyear::avian3d::plugin::*;

pub const TIMESTEP_HZ: f64 = 64.0;
pub const SERVER_PORT: u16 = 5000;

pub fn object_bundle(rigid_body: RigidBody, marker: ObjectMarker) -> impl Bundle {
    let collider = marker.collider();

    (collider, rigid_body, marker)
}

#[derive(Clone)]
pub struct SharedPlugin;

impl Plugin for SharedPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ProtocolPlugin);

        app.add_plugins(lightyear::avian3d::plugin::LightyearAvianPlugin {
            replication_mode: AvianReplicationMode::Position {
                sync_to_transform: false,
            },
            ..default()
        });

        app.add_plugins(
            PhysicsPlugins::default()
                .build()
                .disable::<PhysicsTransformPlugin>()
                .disable::<PhysicsInterpolationPlugin>()
                .disable::<IslandPlugin>()
                .disable::<IslandSleepingPlugin>(),
        );
    }
}
