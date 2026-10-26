use std::net::SocketAddr;

use bevy::{
    ecs::query::QueryFilter,
    prelude::*,
    state::state::States,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};
use bevy_react::prelude::ReactEvents;
use leafwing_input_manager::{action_state::ActionState, input_map::InputMap};
use lightyear::prelude::{client::Client, *};

use super::{
    ClientSettings,
    game::reset_look,
    online::ServerStatuses,
    ui::{
        ConnectionStatusChanged, EditorChanged, LobbyChanged, ScreenChanged, ServerStatusChanged,
        ServerStatusWire, ServersChanged, ShowToast, UiScreen,
    },
};
use crate::config::ServerConfig;
use crate::servers::ServerList;
use crate::{EditorInfo, LobbyInfo, Physics, PhysicsTime, PlayerAction, ServerMode};

#[derive(States, Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum ClientState {
    #[default]
    Menu,
    Connecting,
    Lobby,
    Edit,
    InGame,
    Paused,
    Error,
}

impl ClientState {
    pub fn ui_screen(self) -> UiScreen {
        match self {
            Self::Menu => UiScreen::Menu,
            Self::Connecting => UiScreen::Connecting,
            Self::Lobby => UiScreen::Lobby,
            Self::Edit => UiScreen::Edit,
            Self::InGame => UiScreen::Game,
            Self::Paused => UiScreen::Paused,
            Self::Error => UiScreen::Error,
        }
    }
}

pub trait StateHooks {
    fn start(world: &mut World);
    fn clean(world: &mut World);
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct StateEntity(pub ClientState);

#[derive(Resource, Debug, Clone, Copy)]
pub struct SelectedServer(pub SocketAddr);

impl Default for SelectedServer {
    fn default() -> Self {
        Self(ServerConfig::default().local_addr())
    }
}

#[derive(Resource, Debug, Default, Clone)]
pub struct ConnectionFailure(pub String);

#[derive(Resource, Debug, Default, Clone)]
pub struct StartupWarning(pub Option<String>);

pub(super) struct StatesPlugin;

impl Plugin for StatesPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<ClientState>()
            .init_resource::<SelectedServer>()
            .init_resource::<ConnectionFailure>()
            .init_resource::<StartupWarning>();

        hook_transitions::<ClientState>(app, ClientState::Menu);
        hook_transitions::<ClientState>(app, ClientState::Connecting);
        hook_transitions::<ClientState>(app, ClientState::Lobby);
        hook_transitions::<ClientState>(app, ClientState::Edit);
        hook_transitions::<ClientState>(app, ClientState::InGame);
        hook_transitions::<ClientState>(app, ClientState::Paused);
        hook_transitions::<ClientState>(app, ClientState::Error);

        app.add_systems(
            Update,
            (
                sync_screen.run_if(state_changed::<ClientState>),
                (confirm_connection, follow_server_mode, detect_disconnect).chain(),
                (toggle_pause).run_if(in_play),
                freeze_input.run_if(not(state_changed::<ClientState>)),
            )
                .chain(),
        );

        app.add_systems(
            Update,
            (forward_to_ui, rebroadcast_ui.run_if(every_half_second)).chain(),
        );
    }
}

fn hook_transitions<S: States + StateHooks + Clone>(app: &mut App, state: S) {
    app.add_systems(OnEnter(state.clone()), |world: &mut World| S::start(world));
    app.add_systems(OnExit(state), |world: &mut World| S::clean(world));
}

impl ClientState {
    fn start(self, world: &mut World) {
        match self {
            Self::Menu => menu_start(world),
            Self::Connecting => connecting_start(world),
            Self::Lobby => set_cursor(world, false),
            Self::Edit => {
                set_cursor(world, false);
                set_physics(world, false);
            }
            Self::InGame => in_game_start(world),
            Self::Paused | Self::Error => set_cursor(world, false),
        }
    }

    fn clean(self, world: &mut World) {
        match self {
            Self::Menu => despawn_state(world, Self::Menu),
            Self::Connecting => connecting_clean(world),
            Self::Lobby => {}
            Self::Edit => set_physics(world, true),
            Self::InGame => {}
            Self::Paused => set_cursor(world, true),
            Self::Error => {
                if let Some(mut failure) = world.get_resource_mut::<ConnectionFailure>() {
                    failure.0.clear();
                }
            }
        }
    }
}

impl StateHooks for ClientState {
    fn start(world: &mut World) {
        let state = *world.resource::<State<ClientState>>().get();
        state.start(world);
    }

    fn clean(world: &mut World) {
        let state = *world.resource::<State<ClientState>>().get();
        state.clean(world);
    }
}

fn menu_start(world: &mut World) {
    despawn_all::<With<Client>>(world);

    set_cursor(world, false);
    reset_look(world);

    if let Some(list) = world.get_resource::<ServerList>() {
        let servers = list.all().iter().map(Into::into).collect();
        world.write_message(ServersChanged { servers });
    }
    if let Some(warning) = world.resource_mut::<StartupWarning>().0.take() {
        world.write_message(ShowToast {
            message: warning,
            kind: "error".into(),
        });
    }
}

fn connecting_start(world: &mut World) {
    reset_look(world);
    set_cursor(world, false);

    let server = world.resource::<SelectedServer>().0;
    world.write_message(ConnectionStatusChanged {
        status: "connecting".into(),
        server: server.to_string(),
        message: "Устанавливаем соединение с сервером…".into(),
    });

    spawn_client(world);
}

fn connecting_clean(_world: &mut World) {}

fn in_game_start(world: &mut World) {
    set_cursor(world, true);
    reset_actions(world);
}

fn spawn_client(world: &mut World) {
    let server = world.resource::<SelectedServer>().0;
    let config = world
        .get_resource::<ServerConfig>()
        .cloned()
        .unwrap_or_default();

    let conditioner = LinkConditionerConfig::average_condition().half();
    let entity = world
        .spawn(ClientSettings {
            server_addr: server,
            conditioner: Some(RecvLinkConditioner::new(conditioner)),
            shared: config.shared(),
        })
        .id();
    world.flush();

    if world.get::<Client>(entity).is_some() {
        world.trigger(Connect { entity });
        world.flush();
    } else {
        fail(world, "Не удалось создать сетевой клиент");
    }
}

fn confirm_connection(
    client: Option<Single<(), (With<Client>, With<Connected>)>>,
    state: Res<State<ClientState>>,
    mut commands: Commands,
) {
    if client.is_some() && matches!(state.get(), ClientState::Connecting) {
        commands.set_state(ClientState::Lobby);
    }
}

fn follow_server_mode(
    modes: Query<&ServerMode>,
    state: Res<State<ClientState>>,
    mut commands: Commands,
) {
    for mode in &modes {
        match (mode, state.get()) {
            (ServerMode::Game, ClientState::Lobby | ClientState::Edit) => {
                commands.set_state(ClientState::InGame)
            }
            (ServerMode::Edit, ClientState::Lobby | ClientState::InGame | ClientState::Paused) => {
                commands.set_state(ClientState::Edit)
            }
            (ServerMode::Lobby, ClientState::InGame | ClientState::Paused | ClientState::Edit) => {
                commands.set_state(ClientState::Lobby)
            }
            _ => {}
        }
    }
}

fn detect_disconnect(
    client: Option<Single<&Disconnected, With<Client>>>,
    state: Res<State<ClientState>>,
    selected: Res<SelectedServer>,
    mut failure: ResMut<ConnectionFailure>,
    mut status: MessageWriter<ConnectionStatusChanged>,
    mut commands: Commands,
) {
    let Some(disconnected) = client else {
        return;
    };
    if !matches!(
        state.get(),
        ClientState::Connecting | ClientState::InGame | ClientState::Paused
    ) {
        return;
    }

    let reason = disconnected.reason.to_string();
    failure.0 = reason.clone();
    status.write(ConnectionStatusChanged {
        status: "error".into(),
        server: selected.0.to_string(),
        message: reason,
    });
    commands.set_state(ClientState::Error);
}

fn fail(world: &mut World, reason: &str) {
    world.resource_mut::<ConnectionFailure>().0 = reason.to_string();
    let server = world.resource::<SelectedServer>().0.to_string();
    world.write_message(ConnectionStatusChanged {
        status: "error".into(),
        server,
        message: reason.to_string(),
    });
    world
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::Error);
}

fn in_play(state: Res<State<ClientState>>) -> bool {
    matches!(state.get(), ClientState::InGame | ClientState::Paused)
}

fn toggle_pause(
    state: Res<State<ClientState>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
) {
    if !keys.just_pressed(KeyCode::Escape) {
        return;
    }
    match state.get() {
        ClientState::InGame => commands.set_state(ClientState::Paused),
        ClientState::Paused => commands.set_state(ClientState::InGame),
        _ => {}
    }
}

fn freeze_input(
    cursor: Option<Single<&CursorOptions>>,
    mut actions: Query<&mut ActionState<PlayerAction>, With<InputMap<PlayerAction>>>,
) {
    let grabbed = cursor
        .as_ref()
        .is_some_and(|cursor| cursor.grab_mode != CursorGrabMode::None);
    if grabbed {
        return;
    }
    for mut action in &mut actions {
        action.reset_all();
    }
}

fn sync_screen(state: Res<State<ClientState>>, events: ReactEvents) {
    events.send(&ScreenChanged {
        screen: state.get().ui_screen().name().into(),
    });
}

fn forward_to_ui(
    mut screens: MessageReader<ScreenChanged>,
    mut servers: MessageReader<ServersChanged>,
    mut toasts: MessageReader<ShowToast>,
    mut connections: MessageReader<ConnectionStatusChanged>,
    events: ReactEvents,
) {
    for message in screens.read() {
        events.send(message);
    }
    for message in servers.read() {
        events.send(message);
    }
    for message in toasts.read() {
        events.send(message);
    }
    for message in connections.read() {
        events.send(message);
    }
}

fn rebroadcast_ui(
    state: Res<State<ClientState>>,
    servers: Res<ServerList>,
    statuses: Res<ServerStatuses>,
    lobby: Option<Single<&LobbyInfo>>,
    editor: Option<Single<&EditorInfo>>,
    events: ReactEvents,
) {
    events.send(&ScreenChanged {
        screen: state.get().ui_screen().name().into(),
    });
    let all = servers.all();
    events.send(&ServersChanged {
        servers: all.iter().map(Into::into).collect(),
    });
    events.send(&ServerStatusChanged {
        statuses: all
            .iter()
            .map(|server| server_status(&server.address, &statuses))
            .collect(),
    });
    if let Some(lobby) = lobby {
        events.send(&LobbyChanged {
            players: lobby.players as u32,
            max_players: lobby.max_players as u32,
        });
    }
    if let Some(editor) = editor {
        events.send(&EditorChanged {
            scene: editor.scene.clone(),
            scenes: editor.scenes.clone(),
            message: editor.message.clone(),
            ok: editor.ok,
            can_undo: editor.can_undo,
            can_redo: editor.can_redo,
        });
    }
}

fn server_status(address: &str, statuses: &ServerStatuses) -> ServerStatusWire {
    let reply = statuses.get(address).copied().flatten();
    ServerStatusWire {
        address: address.to_string(),
        online: reply.is_some(),
        players: reply.map(|r| r.players as u32).unwrap_or_default(),
        max_players: reply.map(|r| r.max_players as u32).unwrap_or_default(),
        mode: reply
            .map(|r| match r.mode {
                ServerMode::Lobby => "lobby",
                ServerMode::Game => "game",
                ServerMode::Edit => "edit",
            })
            .unwrap_or_default()
            .to_string(),
    }
}

fn every_half_second(time: Res<Time>) -> bool {
    (time.elapsed_secs() * 2.0) as u64 != ((time.elapsed_secs() - time.delta_secs()) * 2.0) as u64
}

#[allow(dead_code)]
pub fn despawn_state(world: &mut World, state: ClientState) {
    let mut query = world.query_filtered::<Entity, With<StateEntity>>();
    let entities: Vec<Entity> = query.iter(world).collect();
    for entity in entities {
        if world
            .get::<StateEntity>(entity)
            .is_some_and(|e| e.0 == state)
        {
            world.despawn(entity);
        }
    }
}

fn despawn_all<F: QueryFilter>(world: &mut World) {
    let mut query = world.query_filtered::<Entity, F>();
    let entities: Vec<Entity> = query.iter(world).collect();
    for entity in entities {
        world.despawn(entity);
    }
}

fn reset_actions(world: &mut World) {
    let mut query = world.query::<&mut ActionState<PlayerAction>>();
    for mut action in query.iter_mut(world) {
        action.reset_all();
    }
}

pub fn set_physics(world: &mut World, running: bool) {
    let Some(mut time) = world.get_resource_mut::<Time<Physics>>() else {
        return;
    };

    if running {
        time.unpause();
    } else {
        time.pause();
    }
}

pub fn set_cursor(world: &mut World, grabbed: bool) {
    let mut query = world.query_filtered::<&mut CursorOptions, With<PrimaryWindow>>();
    for mut cursor in query.iter_mut(world) {
        if grabbed {
            cursor.grab_mode = CursorGrabMode::Confined;
            cursor.visible = false;
        } else {
            cursor.grab_mode = CursorGrabMode::None;
            cursor.visible = true;
        }
    }
}
