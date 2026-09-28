use doppely::*;

use bevy::prelude::*;
use leafwing_input_manager::action_state::*;
use lightyear::prelude::server::*;
use lightyear::prelude::*;
use std::net::Ipv4Addr;
use std::{io::BufRead, time::Duration};

// use dfdx::prelude::*;
// type Device = Cpu;
// type Model = (Linear<1, 5>, ReLU, Linear<5, 10>);

const SERVER_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), SERVER_PORT);

pub struct ServerPlugin;

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, startup);
        app.add_observer(on_connected);
        app.add_observer(handle_connected);
    }
}

// Event: On new client connected
pub(crate) fn on_connected(trigger: On<Add, LinkOf>, mut commands: Commands) {
    let name = "Client ".to_string() + &trigger.entity.index_u32().to_string();

    commands
        .entity(trigger.entity)
        .insert((ReplicationSender, Name::from(name)));
}

pub(crate) fn handle_connected(
    trigger: On<Add, Connected>,
    query: Query<&RemoteId, With<ClientOf>>,
    mut commands: Commands,
) {
    let Ok(client_id) = query.get(trigger.entity) else {
        return;
    };
    let client_id = client_id.0;
    let entity = commands
        .spawn((
            PlayerId(client_id),
            ActionState::<PlayerAction>::default(),
            Replicate::to_clients(NetworkTarget::All),
            PredictionTarget::to_clients(NetworkTarget::Single(client_id)),
            InterpolationTarget::to_clients(NetworkTarget::AllExceptSingle(client_id)),
            ControlledBy {
                owner: trigger.entity,
                lifetime: Default::default(),
            },
        ))
        .id();

    info!(
        "Create player entity {:?} for client {:?}",
        entity, client_id
    );
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

fn process_input(read: ResMut<ReadLine>, mut state: ResMut<NextState<GameState>>) {
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

        let line = task.join().unwrap_or_default();

        let mut data = line.split(' ');
        let command = data.next().unwrap_or_default();
        let args = data.collect::<Vec<_>>();

        match command {
            "exit" | "quit" | "stop" => {
                bevy::log::info!("Stopping server...");
                std::process::exit(0);
            }
            "players" => {
                bevy::log::info!("Players: ...");
            }
            "mode" => {
                let mode = args.first().cloned().unwrap_or_default();

                match mode.trim() {
                    "editor" => {
                        state.set(GameState::Editor);
                        bevy::log::info!("Switched to mode: editor");
                    }
                    "lobby" => {
                        state.set(GameState::Lobby);
                        bevy::log::info!("Switched to mode: lobby");
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}
