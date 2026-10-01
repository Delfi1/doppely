use bevy::post_process::bloom::*;
use bevy::window::*;
use doppely::*;

use bevy::prelude::*;
use clap::*;
use leafwing_input_manager::plugin::CentralInputStorePlugin;
use leafwing_input_manager::prelude::*;
use leafwing_input_manager::user_input::updating::EnabledInput;
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

        app.add_systems(FixedUpdate, player_movement);
        app.add_systems(
            FixedPostUpdate,
            (render_other_players, render_object, display_debug).chain(),
        );
        app.add_systems(Update, hide_cursor);

        app.add_observer(on_connected);

        app.add_systems(PreUpdate, handle_state);
        app.insert_resource(ServerAddr(self.host));
    }
}

fn on_connected(trigger: On<Add, Controlled>, mut commands: Commands) {
    let input_map = InputMap::new([
        (PlayerAction::Shift, KeyCode::ShiftLeft),
        (PlayerAction::Catch, KeyCode::KeyF),
    ])
    .with_dual_axis(PlayerAction::Move, VirtualDPad::wasd())
    .with_dual_axis(PlayerAction::MouseMove, MouseMove::default());

    commands
        .entity(trigger.entity)
        .insert((input_map, Camera3d::default(), Bloom::default()));
}

fn startup(
    mut commands: Commands,
    mut state: ResMut<NextState<GameState>>,
    server_addr: Res<ServerAddr>,
) {
    state.set(GameState::Loading);

    let mut client = commands.spawn((
        Client,
        LocalAddr(CLIENT_ADDR),
        PeerAddr(server_addr.0),
        Link::default(),
        ReplicationReceiver,
        RawClient,
        UdpIo::default(),
    ));

    client.trigger(Connect::from);
}

fn hide_cursor(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut cursor: Single<Mut<CursorOptions>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        let mode = match cursor.grab_mode {
            CursorGrabMode::None => {
                cursor.visible = false;
                CursorGrabMode::Confined
            }
            _ => {
                cursor.visible = true;
                CursorGrabMode::None
            }
        };
        cursor.grab_mode = mode;
    }

    let mut input = EnabledInput::<MouseMove>::default();
    if cursor.visible {
        input.is_enabled = false;
        commands.insert_resource(input);
    } else {
        commands.insert_resource(input);
    }
}

fn player_movement(
    time: Res<Time>,
    mut player_query: Query<(Mut<Transform>, &ActionState<PlayerAction>), With<PlayerId>>,
) {
    for (transform, action_state) in player_query.iter_mut() {
        shared_movement(action_state, &time, transform);
    }
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
    app.add_plugins((FrameTimeDiagnosticsPlugin::new(64), DefaultPlugins));
    app.add_plugins(ClientPlugins {
        tick_duration: delta,
    });

    app.add_plugins(CentralInputStorePlugin);

    let host = cli.host.parse::<Ipv4Addr>().unwrap_or(Ipv4Addr::LOCALHOST);
    let host = SocketAddr::new(IpAddr::V4(host), SERVER_PORT);

    app.add_plugins(SharedPlugin);
    app.add_plugins(ClientPlugin { host });

    app.run();
}
