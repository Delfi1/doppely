use doppely::*;

use bevy::prelude::*;
use lightyear::connection::server::Start;
use lightyear::prelude::server::ServerPlugins;
use lightyear::prelude::server::ServerUdpIo;
use lightyear::prelude::server::*;
use lightyear::prelude::*;
use std::{io::BufRead, time::Duration};

// use dfdx::prelude::*;
// type Device = Cpu;
// type Model = (Linear<1, 5>, ReLU, Linear<5, 10>);

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

fn main() {
    let mut app = App::new();

    let delta = Duration::from_secs_f64(1.0 / TIMESTEP_HZ);
    app.add_plugins((
        bevy::state::app::StatesPlugin,
        bevy::log::LogPlugin::default(),
        MinimalPlugins.set(bevy::app::ScheduleRunnerPlugin::run_loop(delta)),
        ServerPlugins {
            tick_duration: delta,
        },
    ));

    app.add_plugins((SharedPlugin, ServerPlugin));

    // Command line input
    app.init_resource::<ReadLine>();
    app.add_systems(PreUpdate, process_input);

    app.run();
}

/// Server CLI realisation

#[derive(Resource, Default)]
pub struct ReadLine(pub Option<std::thread::JoinHandle<String>>);

fn process_input(read: ResMut<ReadLine>) {
    let inner = read.into_inner();
    if inner.0.is_none() {
        inner.0 = Some(std::thread::spawn(move || {
            let mut handle = std::io::stdin().lock();

            let mut line = String::new();
            handle.read_line(&mut line).ok();
            line.trim().to_lowercase()
        }));
    }

    if let Some(task) = inner.0.take() {
        if !task.is_finished() {
            inner.0 = Some(task);
            return;
        }

        let command = task.join().unwrap_or_default();

        match command.as_str() {
            "exit" | "quit" | "stop" => {
                bevy::log::info!("Stopping server...");
                std::process::exit(0);
            }
            "players" => {
                bevy::log::info!("Players: ...");
            }
            _ => {}
        }
    }
}
