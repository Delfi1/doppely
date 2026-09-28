use doppely::*;

use bevy::prelude::*;
use clap::*;
use leafwing_input_manager::prelude::*;
use lightyear::prelude::client::*;
use lightyear::prelude::*;
use std::time::Duration;

mod renderer;
use renderer::*;

pub struct ClientPlugin {
    pub host: SocketAddr,
}

const CLIENT_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 4000);

#[derive(Resource, Clone, Deref)]
pub struct ServerAddr(pub SocketAddr);

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, startup);
        app.add_systems(FixedUpdate, apply_movement);

        app.add_systems(FixedPostUpdate, display_debug);
        app.add_observer(on_connected);

        app.add_systems(PreUpdate, handle_state);
        app.insert_resource(ServerAddr(self.host));
    }
}

fn on_connected(trigger: On<Add, Controlled>, mut commands: Commands) {
    bevy::log::info!("Insered controller");
    commands.entity(trigger.entity).insert(Camera3d::default());
}

fn startup(mut commands: Commands, server_addr: Res<ServerAddr>) {
    let input_map = InputMap::new([
        (PlayerAction::Up, KeyCode::KeyW),
        (PlayerAction::Down, KeyCode::KeyS),
        (PlayerAction::Left, KeyCode::KeyA),
        (PlayerAction::Right, KeyCode::KeyD),
        (PlayerAction::Shift, KeyCode::ShiftLeft),
    ]);

    let mut client = commands.spawn((
        Client,
        LocalAddr(CLIENT_ADDR),
        PeerAddr(server_addr.0),
        Link::default(),
        ReplicationReceiver,
        RawClient,
        UdpIo::default(),
        input_map,
    ));

    client.trigger(Connect::from);
}

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Cli {
    #[arg(long, default_value = "127.0.0.1")]
    pub host: String,
}

fn handle_state(mut states: ResMut<NextState<GameState>>, state: Res<GameState>) {
    states.set(state.clone());
}

fn main() {
    let cli = Cli::parse();
    let mut app = App::new();

    let delta = Duration::from_secs_f64(1.0 / TIMESTEP_HZ);
    app.add_plugins(DefaultPlugins);
    app.add_plugins(ClientPlugins {
        tick_duration: delta,
    });

    let host = cli.host.parse::<Ipv4Addr>().unwrap_or(Ipv4Addr::LOCALHOST);
    let host = SocketAddr::new(IpAddr::V4(host), SERVER_PORT);

    app.add_plugins(SharedPlugin);
    app.add_plugins(ClientPlugin { host });

    app.run();
}
