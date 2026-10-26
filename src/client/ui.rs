use bevy::prelude::*;
use bevy_react::prelude::*;
use lightyear::prelude::MessageSender;

use super::editor::EditorSelection;
use super::states::{ClientState, SelectedServer};
use crate::config::ServerConfig;
use crate::scene::SceneComponent;
use crate::servers::{Server, ServerList, ServersPath, display_name, parse_address};
use crate::{EditorAction, EditorChannel, EditorCommand, LobbyChannel, ObjectType, StartGame};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiScreen {
    Menu,
    Connecting,
    Lobby,
    Edit,
    Game,
    Paused,
    Error,
}

impl UiScreen {
    pub fn name(self) -> &'static str {
        match self {
            Self::Menu => "menu",
            Self::Connecting => "connecting",
            Self::Lobby => "lobby",
            Self::Edit => "edit",
            Self::Game => "game",
            Self::Paused => "paused",
            Self::Error => "error",
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase", export)]
pub struct ServerWire {
    pub id: u32,
    pub name: String,
    pub address: String,
    pub local: bool,
}

impl From<&Server> for ServerWire {
    fn from(server: &Server) -> Self {
        Self {
            id: server.id,
            name: server.name.clone(),
            address: server.address.clone(),
            local: server.local,
        }
    }
}

#[react_message(name = "connect")]
#[derive(Debug, Clone, Message)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase", export)]
pub struct ConnectRequest {
    pub id: u32,
}

#[react_message(name = "leave")]
#[derive(Debug, Clone, Message)]

pub struct LeaveRequest {}

#[react_message(name = "resume")]
#[derive(Debug, Clone, Message)]

pub struct ResumeRequest {}

#[react_message(name = "pause")]
#[derive(Debug, Clone, Message)]

pub struct PauseRequest {}

#[react_message(name = "quit")]
#[derive(Debug, Clone, Message)]

pub struct QuitRequest {}

#[react_message(name = "startGame")]
#[derive(Debug, Clone, Message)]
pub struct StartGameRequest {}

#[react_message(name = "saveScene")]
#[derive(Debug, Clone, Message)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase", export)]
pub struct SaveSceneRequest {
    pub name: String,
}

#[react_message(name = "loadScene")]
#[derive(Debug, Clone, Message)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase", export)]
pub struct LoadSceneRequest {
    pub name: String,
}

#[react_message(name = "refreshScenes")]
#[derive(Debug, Clone, Message)]
pub struct RefreshScenesRequest {}

#[react_message(name = "selectObject")]
#[derive(Debug, Clone, Message)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase", export)]
pub struct SelectObjectRequest {
    pub id: u32,
}

#[react_message(name = "moveObject")]
#[derive(Debug, Clone, Message)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase", export)]
pub struct MoveObjectRequest {
    pub id: u32,
    pub position: [f32; 3],
}

#[react_message(name = "setTransform")]
#[derive(Debug, Clone, Message)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase", export)]
pub struct SetTransformRequest {
    pub id: u32,
    pub position: [f32; 3],
    pub rotation: [f32; 3],
    pub size: [f32; 3],
}

#[react_message(name = "undo")]
#[derive(Debug, Clone, Message)]
pub struct UndoRequest {}

#[react_message(name = "redo")]
#[derive(Debug, Clone, Message)]
pub struct RedoRequest {}

#[react_message(name = "removeObject")]
#[derive(Debug, Clone, Message)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase", export)]
pub struct RemoveObjectRequest {
    pub id: u32,
}

#[react_message(name = "removeObjects")]
#[derive(Debug, Clone, Message)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase", export)]
pub struct RemoveObjectsRequest {
    pub ids: Vec<u32>,
}

#[react_message(name = "spawnObject")]
#[derive(Debug, Clone, Message)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase", export)]
pub struct SpawnObjectRequest {
    pub kind: ObjectType,
    pub position: [f32; 3],
}

#[react_message(name = "setComponents")]
#[derive(Debug, Clone, Message)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase", export)]
pub struct SetComponentsRequest {
    pub id: u32,
    pub components: Vec<SceneComponent>,
}

#[react_message(name = "addServer")]
#[derive(Debug, Clone, Message)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase", export)]
pub struct AddServerRequest {
    pub name: String,
    pub address: String,
}

#[react_message(name = "removeServer")]
#[derive(Debug, Clone, Message)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase", export)]
pub struct RemoveServerRequest {
    pub id: u32,
}

#[react_event(name = "ui.screen")]
#[derive(Debug, Clone, Message)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase", export)]
pub struct ScreenChanged {
    pub screen: String,
}

#[react_event(name = "ui.servers")]
#[derive(Debug, Clone, Message)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase", export)]
pub struct ServersChanged {
    pub servers: Vec<ServerWire>,
}

#[react_event(name = "ui.toast")]
#[derive(Debug, Clone, Message)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase", export)]
pub struct ShowToast {
    pub message: String,
    pub kind: String,
}

#[react_event(name = "ui.hud")]
#[derive(Debug, Clone, Message)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase", export)]
pub struct HudUpdated {
    pub server: String,
    pub tick: u32,
    pub fps: u32,
    pub position: Option<[f32; 3]>,
}

#[react_event(name = "ui.connection")]
#[derive(Debug, Clone, Message)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase", export)]
pub struct ConnectionStatusChanged {
    pub status: String,
    pub server: String,
    pub message: String,
}

#[react_event(name = "ui.lobby")]
#[derive(Debug, Clone, Message)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase", export)]
pub struct LobbyChanged {
    pub players: u32,
    pub max_players: u32,
}

#[derive(Debug, Clone, serde::Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase", export)]
pub struct ServerStatusWire {
    pub address: String,
    pub online: bool,
    pub players: u32,
    pub max_players: u32,
    pub mode: String,
}

#[react_event(name = "ui.status")]
#[derive(Debug, Clone, Message)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase", export)]
pub struct ServerStatusChanged {
    pub statuses: Vec<ServerStatusWire>,
}

#[react_event(name = "ui.editor")]
#[derive(Debug, Clone, Message)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase", export)]
pub struct EditorChanged {
    pub scene: String,
    pub scenes: Vec<String>,
    pub message: String,
    pub ok: bool,
    pub can_undo: bool,
    pub can_redo: bool,
}

#[derive(Debug, Clone, serde::Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase", export)]
pub struct SceneObjectWire {
    pub id: u32,
    pub name: String,
    pub kind: String,
    pub position: [f32; 3],
    pub rotation: [f32; 3],
    pub size: [f32; 3],
    pub components: Vec<SceneComponent>,
}

#[react_event(name = "ui.scene")]
#[derive(Debug, Clone, Message)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase", export)]
pub struct SceneChanged {
    pub objects: Vec<SceneObjectWire>,
    pub selected: Vec<u32>,
}

#[react_event(name = "ui.picker")]
#[derive(Debug, Clone, Message)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase", export)]
pub struct EditorPicker {
    pub x: f32,
    pub y: f32,
    pub position: [f32; 3],
}

pub fn register_bindings(app: &mut App) {
    app.add_message::<ScreenChanged>()
        .add_message::<ServersChanged>()
        .add_message::<ShowToast>()
        .add_message::<HudUpdated>()
        .add_message::<ConnectionStatusChanged>()
        .add_message::<LobbyChanged>()
        .add_message::<ServerStatusChanged>()
        .add_message::<EditorChanged>()
        .add_message::<SceneChanged>()
        .add_message::<EditorPicker>()
        .add_react_event::<ScreenChanged>()
        .add_react_event::<ServersChanged>()
        .add_react_event::<ShowToast>()
        .add_react_event::<HudUpdated>()
        .add_react_event::<ConnectionStatusChanged>()
        .add_react_event::<LobbyChanged>()
        .add_react_event::<ServerStatusChanged>()
        .add_react_event::<EditorChanged>()
        .add_react_event::<SceneChanged>()
        .add_react_event::<EditorPicker>()
        .add_react_message::<ConnectRequest>()
        .add_react_message::<LeaveRequest>()
        .add_react_message::<ResumeRequest>()
        .add_react_message::<PauseRequest>()
        .add_react_message::<QuitRequest>()
        .add_react_message::<StartGameRequest>()
        .add_react_message::<SaveSceneRequest>()
        .add_react_message::<LoadSceneRequest>()
        .add_react_message::<RefreshScenesRequest>()
        .add_react_message::<SpawnObjectRequest>()
        .add_react_message::<SelectObjectRequest>()
        .add_react_message::<MoveObjectRequest>()
        .add_react_message::<SetTransformRequest>()
        .add_react_message::<RemoveObjectRequest>()
        .add_react_message::<RemoveObjectsRequest>()
        .add_react_message::<SetComponentsRequest>()
        .add_react_message::<UndoRequest>()
        .add_react_message::<RedoRequest>()
        .add_react_message::<AddServerRequest>()
        .add_react_message::<RemoveServerRequest>();
}

pub fn register_handlers(app: &mut App) {
    app.add_react_handler(on_connect)
        .add_react_handler(on_leave)
        .add_react_handler(on_pause)
        .add_react_handler(on_resume)
        .add_react_handler(on_quit)
        .add_react_handler(on_start_game)
        .add_react_handler(on_save_scene)
        .add_react_handler(on_load_scene)
        .add_react_handler(on_refresh_scenes)
        .add_react_handler(on_spawn_object)
        .add_react_handler(on_select_object)
        .add_react_handler(on_move_object)
        .add_react_handler(on_set_transform)
        .add_react_handler(on_remove_object)
        .add_react_handler(on_remove_objects)
        .add_react_handler(on_set_components)
        .add_react_handler(on_undo)
        .add_react_handler(on_redo)
        .add_react_handler(on_add_server)
        .add_react_handler(on_remove_server);
}

fn on_connect(
    request: On<ConnectRequest>,
    servers: Res<ServerList>,
    state: Res<State<ClientState>>,
    mut selected: ResMut<SelectedServer>,
    mut commands: Commands,
    mut toast: MessageWriter<ShowToast>,
) {
    if matches!(state.get(), ClientState::Connecting) {
        return;
    }

    let Some(server) = servers.get(request.id) else {
        toast.write(ShowToast {
            message: format!("Сервер #{} исчез из списка", request.id),
            kind: "error".into(),
        });
        return;
    };

    selected.0 = server.socket_addr();
    commands.set_state(ClientState::Connecting);
}

fn on_leave(_request: On<LeaveRequest>, state: Res<State<ClientState>>, mut commands: Commands) {
    if !matches!(state.get(), ClientState::Menu) {
        commands.set_state(ClientState::Menu);
    }
}

fn on_pause(_request: On<PauseRequest>, state: Res<State<ClientState>>, mut commands: Commands) {
    if matches!(state.get(), ClientState::InGame) {
        commands.set_state(ClientState::Paused);
    }
}

fn on_resume(_request: On<ResumeRequest>, state: Res<State<ClientState>>, mut commands: Commands) {
    if matches!(state.get(), ClientState::Paused) {
        commands.set_state(ClientState::InGame);
    }
}

fn on_quit(_request: On<QuitRequest>, mut exit: MessageWriter<AppExit>) {
    exit.write(AppExit::Success);
}

fn on_start_game(
    _request: On<StartGameRequest>,
    mut senders: Query<&mut MessageSender<StartGame>>,
) {
    for mut sender in &mut senders {
        sender.send::<LobbyChannel>(StartGame {});
    }
}

fn on_save_scene(
    request: On<SaveSceneRequest>,
    mut senders: Query<&mut MessageSender<EditorCommand>>,
) {
    send_editor(
        &mut senders,
        EditorAction::Save {
            name: request.name.clone(),
        },
    );
}

fn on_load_scene(
    request: On<LoadSceneRequest>,
    mut senders: Query<&mut MessageSender<EditorCommand>>,
) {
    send_editor(
        &mut senders,
        EditorAction::Load {
            name: request.name.clone(),
        },
    );
}

fn send_editor(senders: &mut Query<&mut MessageSender<EditorCommand>>, action: EditorAction) {
    for mut sender in senders.iter_mut() {
        sender.send::<EditorChannel>(EditorCommand {
            action: action.clone(),
        });
    }
}

fn on_refresh_scenes(
    _request: On<RefreshScenesRequest>,
    mut senders: Query<&mut MessageSender<EditorCommand>>,
) {
    send_editor(&mut senders, EditorAction::Refresh);
}

fn on_spawn_object(
    request: On<SpawnObjectRequest>,
    mut senders: Query<&mut MessageSender<EditorCommand>>,
) {
    send_editor(
        &mut senders,
        EditorAction::Spawn {
            kind: request.kind,
            position: Vec3::from_array(request.position),
        },
    );
}

fn on_select_object(
    request: On<SelectObjectRequest>,
    keys: Res<ButtonInput<KeyCode>>,
    mut selection: ResMut<EditorSelection>,
) {
    let additive = keys.pressed(KeyCode::ShiftLeft)
        || keys.pressed(KeyCode::ShiftRight)
        || keys.pressed(KeyCode::ControlLeft)
        || keys.pressed(KeyCode::ControlRight);

    selection.select_from_ui(request.id, additive);
}

fn on_move_object(
    request: On<MoveObjectRequest>,
    mut senders: Query<&mut MessageSender<EditorCommand>>,
) {
    send_editor(
        &mut senders,
        EditorAction::Move {
            id: request.id,
            position: Vec3::from_array(request.position),
        },
    );
}

fn on_set_transform(
    request: On<SetTransformRequest>,
    mut senders: Query<&mut MessageSender<EditorCommand>>,
) {
    send_editor(
        &mut senders,
        EditorAction::SetTransform {
            id: request.id,
            position: Vec3::from_array(request.position),
            rotation: Vec3::from_array(request.rotation),
            size: Vec3::from_array(request.size),
        },
    );
}

fn on_undo(_request: On<UndoRequest>, mut senders: Query<&mut MessageSender<EditorCommand>>) {
    send_editor(&mut senders, EditorAction::Undo);
}

fn on_redo(_request: On<RedoRequest>, mut senders: Query<&mut MessageSender<EditorCommand>>) {
    send_editor(&mut senders, EditorAction::Redo);
}

fn on_remove_object(
    request: On<RemoveObjectRequest>,
    mut selection: ResMut<EditorSelection>,
    mut senders: Query<&mut MessageSender<EditorCommand>>,
) {
    selection.remove(request.id);
    send_editor(&mut senders, EditorAction::Remove { id: request.id });
}

fn on_remove_objects(
    request: On<RemoveObjectsRequest>,
    mut selection: ResMut<EditorSelection>,
    mut senders: Query<&mut MessageSender<EditorCommand>>,
) {
    for id in &request.ids {
        selection.remove(*id);
    }
    send_editor(
        &mut senders,
        EditorAction::RemoveMany {
            ids: request.ids.clone(),
        },
    );
}

fn on_set_components(
    request: On<SetComponentsRequest>,
    mut senders: Query<&mut MessageSender<EditorCommand>>,
) {
    send_editor(
        &mut senders,
        EditorAction::SetComponents {
            id: request.id,
            components: request.components.clone(),
        },
    );
}

fn on_add_server(
    request: On<AddServerRequest>,
    config: Res<ServerConfig>,
    path: Res<ServersPath>,
    mut servers: ResMut<ServerList>,
    mut broadcast: MessageWriter<ServersChanged>,
    mut toast: MessageWriter<ShowToast>,
) {
    let address = match parse_address(&request.address, &config) {
        Ok(address) => address,
        Err(error) => {
            toast.write(ShowToast {
                message: error,
                kind: "error".into(),
            });
            return;
        }
    };

    let name = display_name(&request.name, &address);
    if !servers.insert(name, address.clone()) {
        toast.write(ShowToast {
            message: format!("Сервер {address} уже есть в списке"),
            kind: "error".into(),
        });
        return;
    }

    if let Some(error) = servers.save(&path) {
        toast.write(ShowToast {
            message: error,
            kind: "error".into(),
        });
    }
    broadcast.write(ServersChanged {
        servers: servers.all().iter().map(Into::into).collect(),
    });
}

fn on_remove_server(
    request: On<RemoveServerRequest>,
    path: Res<ServersPath>,
    mut servers: ResMut<ServerList>,
    mut broadcast: MessageWriter<ServersChanged>,
    mut toast: MessageWriter<ShowToast>,
) {
    if !servers.remove(request.id) {
        return;
    }

    if let Some(error) = servers.save(&path) {
        toast.write(ShowToast {
            message: error,
            kind: "error".into(),
        });
    }
    broadcast.write(ServersChanged {
        servers: servers.all().iter().map(Into::into).collect(),
    });
}
