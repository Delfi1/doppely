use doppely::*;

use bevy::prelude::*;
use leafwing_input_manager::action_state::*;
use lightyear::prelude::server::*;
use lightyear::prelude::*;
use std::{io::BufRead, time::Duration};

mod train;
use train::*;

const SERVER_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), SERVER_PORT);

pub struct ServerPlugin;

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TrainPlugin);

        app.add_systems(Startup, startup);
        app.add_systems(FixedUpdate, (player_rotation, player_movement).chain());

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
            Position(Vec3::new(0.0, 0.0, 4.0)),
            Replicate::to_clients(NetworkTarget::All),
            PredictionTarget::to_clients(NetworkTarget::Single(client_id)),
            InterpolationTarget::to_clients(NetworkTarget::AllExceptSingle(client_id)),
            ControlledBy {
                owner: trigger.entity,
                lifetime: Lifetime::Persistent,
            },
            DisableReplicateHierarchy,
            object_bundle(RigidBody::Dynamic, ObjectMarker::Player),
            player_physics(),
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

    // Spawn test object - cube
    commands.spawn((
        Name::new("Cube"),
        Position(Vec3::new(0.0, 0.0, 0.0)),
        Glowing {
            color: Srgba::new(100.0, 20.0, 200.0, 1.0),
        },
        Light {
            radius: 10.0,
            color: Srgba::new(100.0, 20.0, 200.0, 1.0),
            intensity: 100_000.0,
        },
        Replicate::to_clients(NetworkTarget::All),
        object_bundle(RigidBody::Dynamic, ObjectMarker::Cube),
    ));

    commands.spawn((
        Name::new("Floor"),
        Position(Vec3::new(0.0, -2.2, 0.0)),
        Replicate::to_clients(NetworkTarget::All),
        object_bundle(RigidBody::Static, ObjectMarker::Floor),
    ));
    Ok(())
}

fn player_rotation(
    time: Res<Time>,
    input_timeline: Option<SyncedLocalTimeline>,
    mut player_query: Query<
        (Has<Predicted>, &ActionState<PlayerAction>, &mut Rotation),
        With<PlayerId>,
    >,
) {
    let client_is_synced = input_timeline.is_some();
    for (is_predicted, action, mut rotation) in player_query.iter_mut() {
        if is_predicted && !client_is_synced {
            continue;
        }
        shared_rotation(action, &time, &mut rotation);
    }
}

fn player_movement(
    time: Res<Time>,
    input_timeline: Option<SyncedLocalTimeline>,
    mut player_query: Query<
        (
            Has<Predicted>,
            &ActionState<PlayerAction>,
            &ComputedMass,
            Forces,
        ),
        With<PlayerId>,
    >,
) {
    let client_is_synced = input_timeline.is_some();
    for (is_predicted, action, mass, forces) in player_query.iter_mut() {
        if is_predicted && !client_is_synced {
            continue;
        }
        shared_movement(action, &time, mass, forces);
    }
}

fn main() {
    let mut app = App::new();

    let delta = Duration::from_secs_f64(1.0 / TIMESTEP_HZ);
    app.add_plugins((
        TransformPlugin,
        bevy::input::InputPlugin,
        AssetPlugin::default(),
        bevy::state::app::StatesPlugin,
        bevy::app::TerminalCtrlCHandlerPlugin,
        bevy::log::LogPlugin::default(),
        MinimalPlugins.set(bevy::app::ScheduleRunnerPlugin::run_loop(delta)),
        ServerPlugins {
            tick_duration: delta,
        },
    ));

    app.init_resource::<Assets<Mesh>>()
        .add_message::<AssetEvent<Mesh>>();

    app.add_plugins((SharedPlugin, ServerPlugin));

    // Command line input
    app.init_resource::<ReadLine>();
    app.add_systems(PreUpdate, process_input);

    app.run();
}

/// Server CLI realisation

#[derive(Resource, Default)]
pub struct ReadLine(pub Option<std::thread::JoinHandle<String>>);

//mut state: ResMut<NextState<GameState>>
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
                let _mode = args.first().cloned().unwrap_or_default();

                // match mode.trim() {
                //     "editor" => {
                //         state.set(GameState::Editor);
                //         bevy::log::info!("Switched to mode: editor");
                //     }
                //     "lobby" => {
                //         state.set(GameState::Lobby);
                //         bevy::log::info!("Switched to mode: lobby");
                //     }
                //     _ => {}
                // }
            }
            v => {
                bevy::log::warn!("Unknown command: {}", v);
            }
        }
    }
}
