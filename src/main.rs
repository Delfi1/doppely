//! Сервер управялет ИИ агентами, занимается их обучением, а так-же работает с подключениями игроков.
//! Для создания нейросети используется крейт [dfdx](https://docs.rs/dfdx/latest/dfdx/)
//! Для игровой логики используется движок [bevy](https://docs.rs/bevy/latest/bevy/)
//! Для клиент-серверного взаимодействия используется [lightyear](https://docs.rs/lightyear/latest/lightyear/)

#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

mod shared;

#[cfg(feature = "client")]
mod client;
#[cfg(feature = "server")]
mod server;

use crate::shared::{SharedPlugin, TIMESTEP_HZ};
use bevy::prelude::*;
use core::time::Duration;
use lightyear::connection::server::Start;
use lightyear::prelude::client::ClientPlugins;
use lightyear::prelude::server::ServerPlugins;
use lightyear::prelude::server::ServerUdpIo;
use lightyear::prelude::*;

use bevy::prelude::*;

fn main() {
    let mut app = App::new();

    #[cfg(feature = "client")]
    {
        app.add_plugins(DefaultPlugins);
        app.add_plugins(ClientPlugins {
            tick_duration: Duration::from_secs_f64(1.0 / TIMESTEP_HZ),
        });
    }

    #[cfg(feature = "server")]
    {
        let delta = Duration::from_secs_f64(1.0 / TIMESTEP_HZ);
        app.add_plugins((
            DefaultPlugins.set(bevy::app::ScheduleRunnerPlugin::run_loop(delta)),
            ServerPlugins {
                tick_duration: Duration::from_secs_f64(1.0 / TIMESTEP_HZ),
            },
        ));
    }

    app.add_plugins(SharedPlugin);

    #[cfg(feature = "client")]
    app.add_plugins(client::ClientPlugin);
    #[cfg(feature = "server")]
    app.add_plugins(server::ServerPlugin);

    app.run();
}
