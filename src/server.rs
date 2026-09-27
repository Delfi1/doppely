use crate::shared::*;
use bevy::prelude::*;
use dfdx::prelude::*;
use lightyear::prelude::client::*;
use lightyear::prelude::server::*;
use lightyear::prelude::*;
use std::time::Duration;

use dfdx::prelude::*;

type Device = Cpu;

type Model = (Linear<1, 5>, ReLU, Linear<5, 10>);

pub struct ServerPlugin;

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, startup);
        app.add_systems(Update, tick_player);
        app.add_observer(handle_new_client);
    }
}

fn handle_new_client(trigger: On<Add, Connected>, mut commands: Commands) {
    commands.entity(trigger.entity).insert(ReplicationSender);

    commands.spawn((
        PlayerPosition::default(),
        Replicate::to_clients(NetworkTarget::All),
    ));

    info!("New client connected: {}", trigger.entity);
}

fn tick_player(mut players: Query<&mut PlayerPosition>, time: Res<Time>) {
    let delta = time.delta_secs();
    for mut player in players.iter_mut() {
        player.z += 2.0 * delta;

        if player.z > 5.0 {
            player.z = -5.0;
        }
    }
}

fn startup(mut commands: Commands) -> Result {
    let server = commands
        .spawn((RawServer, LocalAddr(SERVER_ADDR), ServerUdpIo::default()))
        .id();

    commands.trigger(Start { entity: server });
    Ok(())
}
