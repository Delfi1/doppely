//! Клиент Doppely: главное меню со списком серверов, подключение, игра с HUD и
//! Esc-меню.
//!
//! UI рисует React-приложение из `ui/`, а состояние и сеть живут здесь: экран
//! считается из [`ClientState`] (см. [`states`]), данные ходят через
//! [`ui`], серверы хранятся в [`crate::servers`].

mod game;
mod hud;
mod renderer;
mod states;
mod ui;

use std::net::*;

use bevy::{
    diagnostic::FrameTimeDiagnosticsPlugin,
    ecs::{lifecycle::HookContext, world::DeferredWorld},
    input::keyboard::KeyCode,
    prelude::*,
};
use bevy_react::prelude::*;
use doppely::*;
use leafwing_input_manager::{
    input_map::InputMap, plugin::InputManagerPlugin, user_input::VirtualDPad,
};
use lightyear::{
    netcode::{NetcodeClient, client_plugin::NetcodeConfig},
    prelude::{client::ClientPlugins, *},
};

use game::GamePlugin;
use renderer::{RenderingPlugin, controller_camera};
use states::{ClientState, SelectedServer, StartupWarning, StatesPlugin};
use ui::register_bindings;

const CLIENT_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), CLIENT_PORT);

/// Подключение к серверу: `ClientSettings` создаёт netcode-клиент, сокет и
/// адреса. Сущность живёт ровно один сеанс связи (см. `states::connecting_clean`).
#[derive(Component, Clone, Debug)]
#[component(on_add = ClientSettings::on_add)]
pub struct ClientSettings {
    pub server_addr: SocketAddr,
    pub conditioner: Option<RecvLinkConditioner>,
    pub shared: SharedSettings,
}

impl ClientSettings {
    fn on_add(mut world: DeferredWorld, context: HookContext) {
        let entity = context.entity;
        world.commands().queue(move |world: &mut World| -> Result {
            world.insert_resource(PredictionManager::default());

            let mut entity_mut = world.entity_mut(entity);
            let settings = entity_mut.take::<ClientSettings>().unwrap();
            entity_mut.insert((
                ReplicationReceiver,
                Link::default().with_conditioner(settings.conditioner.clone()),
                LocalAddr(CLIENT_ADDR),
                PeerAddr(settings.server_addr),
                Name::from("Client"),
            ));

            let auth = Authentication::Manual {
                server_addr: settings.server_addr,
                client_id: rand::random(),
                private_key: settings.shared.private_key,
                protocol_id: settings.shared.protocol_id,
            };
            let netcode_config = NetcodeConfig {
                client_timeout_secs: 3,
                token_expire_secs: -1,
                ..default()
            };
            entity_mut.insert(NetcodeClient::new(auth, netcode_config)?);
            entity_mut.insert(UdpIo::default());

            Ok(())
        });
    }
}

fn setup_controller(trigger: On<Add<Controlled>>, mut commands: Commands) {
    let entity = trigger.entity;
    let input_map = InputMap::new([
        (PlayerAction::Shift, KeyCode::ShiftLeft),
        (PlayerAction::Catch, KeyCode::KeyF),
    ])
    .with_dual_axis(PlayerAction::Move, VirtualDPad::wasd());

    commands
        .entity(entity)
        .insert((input_map, InheritedVisibility::VISIBLE))
        .with_child(controller_camera());
}

fn main() {
    if std::env::args().nth(1).as_deref() == Some("--export-bindings") {
        export_bindings();
        return;
    }

    let mut app = App::new();

    let delta = Duration::from_secs_f64(1.0 / TIMESTEP_HZ);
    app.add_plugins((
        FrameTimeDiagnosticsPlugin::new(64),
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Doppely".into(),
                ..default()
            }),
            ..default()
        }),
    ));
    app.add_plugins(ClientPlugins {
        tick_duration: delta,
    });

    app.add_plugins(SharedPlugin);
    load_servers(&mut app);

    app.add_plugins(ClientPlugin);
    // Ввод игрока рассылается на сервер из `ActionState`, который наполняет
    // именно этот плагин; без него игрок не двигается.
    app.add_plugins(InputManagerPlugin::<PlayerAction>::default());

    // `--connect <addr>` пропускает меню: удобно для отладки без кликов.
    if let Some(server) = connect_arg() {
        app.insert_resource(SelectedServer(server));
        app.insert_state(ClientState::Connecting);
    }

    app.run();
}

fn export_bindings() {
    let path = std::env::args()
        .nth(2)
        .unwrap_or_else(|| "ui/src/bevy.ts".into());

    let mut app = App::new();
    ReactPlugins::register_bindings(&mut app);
    register_bindings(&mut app);
    if let Err(error) = app.export_react_typescript(&path) {
        eprintln!("не удалось записать {path}: {error}");
        std::process::exit(1);
    }
}

/// Читает `assets/servers.json` до старта UI: меню должно получить список в
/// первом же кадре, иначе на мгновение показывается пустая панель.
fn load_servers(app: &mut App) {
    let path = crate::servers::ServersPath::resolve();
    let (entries, warning) = path.load();
    info!("Список серверов: {}", path.write_path().display());
    if let Some(warning) = &warning {
        warn!("{warning}");
    }

    app.insert_resource(crate::servers::ServerList::with_servers(entries))
        .insert_resource(path)
        .insert_resource(StartupWarning(warning));
}

fn connect_arg() -> Option<SocketAddr> {
    let mut args = std::env::args().skip(1);
    let flag = args.next()?;
    if flag != "--connect" {
        return None;
    }
    let value = args.next()?;
    match crate::servers::parse_address(&value) {
        Ok(addr) => addr.parse().ok(),
        Err(error) => {
            warn!("--connect {value}: {error}");
            None
        }
    }
}

pub struct ClientPlugin;

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(GamePlugin);
        app.add_observer(setup_controller);

        app.add_plugins(RenderingPlugin);
        app.add_plugins(StatesPlugin);
        app.add_plugins(hud::HudPlugin);

        ReactPlugins::register_bindings(app);
        register_bindings(app);
        ui::register_handlers(app);

        app.add_plugins(
            ReactPlugins.set(
                ReactUiPlugin::new("ui/dist/app.js").default_font("fonts/FiraSans-Regular.ttf"),
            ),
        );
    }
}
