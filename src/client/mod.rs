use bevy::input::mouse::*;
use bevy::post_process::bloom::*;
use bevy::window::*;
use doppely::*;

use bevy::prelude::*;
use clap::*;
use leafwing_input_manager::plugin::InputManagerSystem;
use leafwing_input_manager::prelude::*;
use lightyear::input::client::InputSystems;
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
        app.add_systems(
            FixedPreUpdate,
            update_cursor
                .chain()
                .before(InputSystems::BufferClientInputs)
                .in_set(InputManagerSystem::ManualControl),
        );

        app.add_systems(
            FixedPostUpdate,
            (render_other_players, render_cubes, display_debug).chain(),
        );
        app.add_systems(Update, hide_cursor);

        app.add_observer(on_connected);

        app.add_systems(PreUpdate, handle_state);
        app.insert_resource(ServerAddr(self.host));
    }
}

fn on_connected(trigger: On<Add, Controlled>, mut commands: Commands) {
    let input_map = InputMap::new([
        (PlayerAction::Forward, KeyCode::KeyW),
        (PlayerAction::Backward, KeyCode::KeyS),
        (PlayerAction::Left, KeyCode::KeyA),
        (PlayerAction::Right, KeyCode::KeyD),
        (PlayerAction::Shift, KeyCode::ShiftLeft),
    ]);

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

fn hide_cursor(keyboard: Res<ButtonInput<KeyCode>>, mut cursor: Single<Mut<CursorOptions>>) {
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
}

fn update_cursor(
    mut moved: MessageReader<MouseMotion>,
    time: Res<Time>,
    mut action_state: Single<&mut ActionState<PlayerAction>, With<InputMap<PlayerAction>>>,
    cursor: Single<Ref<CursorOptions>>,
) {
    let time_delta = time.delta_secs();

    let mut delta = Vec2::ZERO;
    for motion in moved.read() {
        delta += motion.delta;
    }

    if delta == Vec2::ZERO || cursor.visible {
        return;
    }

    action_state.set_axis_pair(&PlayerAction::MoveCursor, delta * time_delta);
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

    let host = cli.host.parse::<Ipv4Addr>().unwrap_or(Ipv4Addr::LOCALHOST);
    let host = SocketAddr::new(IpAddr::V4(host), SERVER_PORT);

    app.add_plugins(SharedPlugin);
    app.add_plugins(ClientPlugin { host });

    app.run();
}
