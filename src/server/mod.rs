use bevy::ecs::{lifecycle::HookContext, world::DeferredWorld};
use doppely::*;
use leafwing_input_manager::action_state::ActionState;
use lightyear::{
    connection::{client_of::ClientOf, server::Start},
    prelude::{server::*, *},
};
use std::io::BufRead;

const SERVER_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), SERVER_PORT);
const SEND_INTERVAL: Duration = Duration::from_millis(16);

pub struct ServerPlugin;

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(handle_new);
        app.add_observer(on_connected);

        app.add_systems(Startup, setup);
        app.add_systems(Startup, start);
    }
}

#[derive(Component, Debug)]
#[component(on_add = ServerSettings::on_add)]
pub struct ServerSettings {
    pub conditioner: Option<RecvLinkConditioner>,
    pub shared: SharedSettings,
}

impl ServerSettings {
    fn on_add(mut world: DeferredWorld, context: HookContext) {
        let entity = context.entity;
        world.commands().queue(move |world: &mut World| -> Result {
            let mut entity_mut = world.entity_mut(entity);
            let settings = entity_mut.take::<ServerSettings>().unwrap();
            entity_mut.insert((
                Name::from("Server"),
                Endpoint::new(settings.conditioner.clone()),
            ));

            entity_mut.insert(NetcodeServer::new(NetcodeConfig {
                protocol_id: settings.shared.protocol_id,
                private_key: settings.shared.private_key,
                server_addr_check: false,
                ..Default::default()
            }));
            entity_mut.insert((LocalAddr(SERVER_ADDR), ServerUdpIo::default()));
            Ok(())
        });
    }
}

fn main() {
    let mut app = App::new();

    app.insert_resource(ReplicationMetadata::new(SEND_INTERVAL));
    let delta = Duration::from_secs_f64(1.0 / TIMESTEP_HZ);
    app.add_plugins((
        TransformPlugin,
        bevy::input::InputPlugin,
        AssetPlugin::default(),
        bevy::state::app::StatesPlugin,
        bevy::app::TerminalCtrlCHandlerPlugin,
        bevy::log::LogPlugin::default(),
        MinimalPlugins,
        ServerPlugins {
            tick_duration: delta,
        },
    ));

    app.init_resource::<Assets<Mesh>>()
        .add_message::<AssetEvent<Mesh>>();

    app.add_plugins(SharedPlugin);

    let conditioner = LinkConditionerConfig::average_condition().half();
    app.world_mut().spawn(ServerSettings {
        conditioner: Some(RecvLinkConditioner::new(conditioner.clone())),
        shared: SHARED_SETTINGS,
    });

    app.add_plugins(ServerPlugin);

    // Command line input
    app.init_resource::<ReadLine>();
    app.add_systems(PreUpdate, process_input);

    app.run();
}

/// Test world setup
fn setup(mut commands: Commands) {
    commands.spawn((
        Name::new("Floor"),
        ObjectMarker::Floor,
        Position::new(Vec3::ZERO),
        Replicate::to_clients(NetworkTarget::All),
    ));

    commands.spawn((
        Name::new("Crate"),
        ObjectMarker::Crate,
        Glowing {
            color: Srgba::new(100.0, 20.0, 200.0, 1.0),
        },
        Light {
            radius: 10.0,
            color: Srgba::new(100.0, 20.0, 200.0, 1.0),
            intensity: 100_000.0,
        },
        Position::new(Vec3::new(1.0, 2.0, 0.0)),
        Replicate::to_clients(NetworkTarget::All),
        PredictionTarget::to_clients(NetworkTarget::All),
    ));
}

/// Start sever listening
fn start(mut commands: Commands, server: Single<Entity, With<Server>>) {
    commands.trigger(Start {
        entity: server.into_inner(),
    });
}

fn handle_new(trigger: On<Add, LinkOf>, mut commands: Commands) {
    commands.entity(trigger.entity).insert(ReplicationSender);
}

fn on_connected(
    trigger: On<Add, Connected>,
    query: Query<&RemoteId, With<ClientOf>>,
    mut commands: Commands,
) {
    let Ok(client_id) = query.get(trigger.entity) else {
        return;
    };
    let client_id = client_id.0;

    commands.spawn((
        Name::new("Player"),
        ObjectMarker::Player,
        ActionState::<PlayerAction>::default(),
        Position(Vec3::new(0.0, 2.0, 4.0)),
        PlayerId(client_id),
        DisableReplicateHierarchy,
        Replicate::to_clients(NetworkTarget::All),
        PredictionTarget::to_clients(NetworkTarget::All),
        InterpolationTarget::to_clients(NetworkTarget::AllExceptSingle(client_id)),
        ControlledBy {
            owner: trigger.entity,
            lifetime: Default::default(),
        },
    ));
}

/// Server CLI realisation

#[derive(Resource, Default)]
pub struct ReadLine(pub Option<std::thread::JoinHandle<String>>);

fn process_input(
    read: ResMut<ReadLine>,
    objects: Query<(&ObjectType, &Position)>,
    players: Query<Entity, With<PlayerId>>,
) {
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
        let _args = data.collect::<Vec<_>>();

        match command {
            "exit" | "quit" | "stop" => {
                bevy::log::info!("Stopping server...");
                std::process::exit(0);
            }
            "players" => {
                let count = players.iter().count();
                bevy::log::info!("Players: {}", count);
            }
            "debug" => {
                for (ty, pos) in objects {
                    bevy::log::info!("{:?} Position: {:.3}", ty, pos.0);
                }
            }
            v => {
                bevy::log::warn!("Unknown command: \"{}\"", v);
            }
        }
    }
}
