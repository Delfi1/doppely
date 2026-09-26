use crate::shared::*;
use bevy::prelude::*;
use dfdx::prelude::*;
use lightyear::prelude::client::*;
use lightyear::prelude::server::*;
use lightyear::prelude::*;
use std::time::Duration;

pub struct ServerPlugin;

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, startup);
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

fn startup(mut commands: Commands) -> Result {
    let server = commands
        .spawn((RawServer, LocalAddr(SERVER_ADDR), ServerUdpIo::default()))
        .id();

    commands.trigger(Start { entity: server });
    Ok(())
}
