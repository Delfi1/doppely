mod game;
mod renderer;

use bevy::ecs::{lifecycle::HookContext, world::DeferredWorld};
use bevy_lunex::prelude::*;
use leafwing_input_manager::{input_map::InputMap, user_input::VirtualDPad};
use lightyear::{
    netcode::{NetcodeClient, client_plugin::NetcodeConfig},
    prelude::{client::ClientPlugins, *},
};

use doppely::*;
use game::*;
use renderer::*;

const CLIENT_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), CLIENT_PORT);

pub struct ClientPlugin;

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(GamePlugin);
        app.add_observer(setup_controller);

        app.add_plugins(RenderingPlugin);
    }
}

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

fn setup_controller(trigger: On<Add, Controlled>, mut commands: Commands) {
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
    let mut app = App::new();

    let delta = Duration::from_secs_f64(1.0 / TIMESTEP_HZ);
    app.add_plugins((FrameTimeDiagnosticsPlugin::new(64), DefaultPlugins));
    app.add_plugins(ClientPlugins {
        tick_duration: delta,
    });

    app.add_plugins(SharedPlugin);

    let conditioner = LinkConditionerConfig::average_condition().half();
    // TODO:
    let server_addr = SocketAddr::new(Ipv4Addr::new(192, 168, 3, 5).into(), SERVER_PORT);
    app.world_mut().spawn(ClientSettings {
        server_addr,
        conditioner: Some(RecvLinkConditioner::new(conditioner.clone())),
        shared: SHARED_SETTINGS,
    });

    app.add_plugins((ClientPlugin, UiLunexPlugins));
    app.add_systems(Startup, connect);
    app.run();
}

fn connect(mut commands: Commands, client: Single<Entity, With<Client>>) {
    commands.trigger(Connect {
        entity: client.into_inner(),
    });
}
