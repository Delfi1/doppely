//! Обмен данными UI (React) и клиента (Bevy): типизированные сообщения React →
//! Rust и события Rust → React.
//!
//! Имена каналов и формы структур здесь — единственный источник истины:
//! `ui/src/bevy.ts` генерируется из этого файла (`npm run bevy:generate`), так
//! что TypeScript и serde не могут разойтись.

use bevy::prelude::*;
use bevy_react::prelude::*;

use super::states::{ClientState, SelectedServer};
use crate::servers::{Server, ServerList, ServersPath, display_name, parse_address};

/// Экран, который показывает UI. Считается из состояния клиента (подключение,
/// пауза) и рассылается в React при изменении.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiScreen {
    /// Главное меню со списком серверов.
    Menu,
    /// Ждём подтверждения связи с сервером.
    Connecting,
    /// Игра: HUD поверх 3D-сцены.
    Game,
    /// Esc-меню поверх приостановленного HUD.
    Paused,
    /// Связь потеряна или сервер отверг подключение.
    Error,
}

impl UiScreen {
    /// Имя экрана для React (совпадает с `Screen` в `ui/src/bevy.ts`).
    pub fn name(self) -> &'static str {
        match self {
            Self::Menu => "menu",
            Self::Connecting => "connecting",
            Self::Game => "game",
            Self::Paused => "paused",
            Self::Error => "error",
        }
    }
}

/// Сервер в том виде, в котором его потребляет UI.
#[derive(Debug, Clone, serde::Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase", export)]
pub struct ServerWire {
    pub id: u32,
    pub name: String,
    pub address: String,
    /// Localhost: неудаляемый и всегда первый.
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

// --- React → Rust -------------------------------------------------------------

/// Подключиться к серверу из списка.
#[react_message(name = "connect")]
#[derive(Debug, Clone, Message)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase", export)]
pub struct ConnectRequest {
    pub id: u32,
}

/// Отключиться и вернуться в главное меню (из Esc-меню или экрана ошибки).
#[react_message(name = "leave")]
#[derive(Debug, Clone, Message)]

pub struct LeaveRequest {}

/// Продолжить игру (закрыть Esc-меню).
#[react_message(name = "resume")]
#[derive(Debug, Clone, Message)]

pub struct ResumeRequest {}

/// Приостановить игру (Esc в игре, клик по кнопке паузы в HUD).
#[react_message(name = "pause")]
#[derive(Debug, Clone, Message)]

pub struct PauseRequest {}

/// Выйти из приложения.
#[react_message(name = "quit")]
#[derive(Debug, Clone, Message)]

pub struct QuitRequest {}

/// Добавить сервер в список. Адрес нормализуется на стороне Rust.
#[react_message(name = "addServer")]
#[derive(Debug, Clone, Message)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase", export)]
pub struct AddServerRequest {
    pub name: String,
    pub address: String,
}

/// Удалить добавленный сервер.
#[react_message(name = "removeServer")]
#[derive(Debug, Clone, Message)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase", export)]
pub struct RemoveServerRequest {
    pub id: u32,
}

// --- Rust → React -------------------------------------------------------------

/// Текущий экран клиента.
#[react_event(name = "ui.screen")]
#[derive(Debug, Clone, Message)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase", export)]
pub struct ScreenChanged {
    /// `UiScreen::name`: `"menu" | "connecting" | "game" | "paused" | "error"`.
    pub screen: String,
}

/// Актуальный список серверов (после загрузки или изменения).
#[react_event(name = "ui.servers")]
#[derive(Debug, Clone, Message)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase", export)]
pub struct ServersChanged {
    pub servers: Vec<ServerWire>,
}

/// Сообщение для тоста в UI.
#[react_event(name = "ui.toast")]
#[derive(Debug, Clone, Message)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase", export)]
pub struct ShowToast {
    pub message: String,
    /// `"error" | "info"`.
    pub kind: String,
}

/// Снимок HUD: сеть, кадры, игрок.
#[react_event(name = "ui.hud")]
#[derive(Debug, Clone, Message)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase", export)]
pub struct HudUpdated {
    /// Адрес сервера, к которому подключены.
    pub server: String,
    /// Сетевой тик (локальный таймлайн).
    pub tick: u32,
    pub fps: u32,
    /// Позиция локального игрока; `None`, пока сервер её не реплицировал.
    pub position: Option<[f32; 3]>,
}

/// Данные экранов подключения и ошибки.
#[react_event(name = "ui.connection")]
#[derive(Debug, Clone, Message)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase", export)]
pub struct ConnectionStatusChanged {
    /// `"connecting" | "error"`.
    pub status: String,
    pub server: String,
    pub message: String,
}

/// Регистрирует все каналы UI. Вызывается и в рантайме, и генератором
/// `ui/src/bevy.ts`, чтобы биндинги не расходились.
pub fn register_bindings(app: &mut App) {
    app.add_message::<ScreenChanged>()
        .add_message::<ServersChanged>()
        .add_message::<ShowToast>()
        .add_message::<HudUpdated>()
        .add_message::<ConnectionStatusChanged>()
        .add_react_event::<ScreenChanged>()
        .add_react_event::<ServersChanged>()
        .add_react_event::<ShowToast>()
        .add_react_event::<HudUpdated>()
        .add_react_event::<ConnectionStatusChanged>()
        .add_react_message::<ConnectRequest>()
        .add_react_message::<LeaveRequest>()
        .add_react_message::<ResumeRequest>()
        .add_react_message::<PauseRequest>()
        .add_react_message::<QuitRequest>()
        .add_react_message::<AddServerRequest>()
        .add_react_message::<RemoveServerRequest>();
}

/// Подключает обработчики намерений из UI.
pub fn register_handlers(app: &mut App) {
    app.add_react_handler(on_connect)
        .add_react_handler(on_leave)
        .add_react_handler(on_pause)
        .add_react_handler(on_resume)
        .add_react_handler(on_quit)
        .add_react_handler(on_add_server)
        .add_react_handler(on_remove_server);
}

// --- Обработчики --------------------------------------------------------------

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

fn on_add_server(
    request: On<AddServerRequest>,
    path: Res<ServersPath>,
    mut servers: ResMut<ServerList>,
    mut broadcast: MessageWriter<ServersChanged>,
    mut toast: MessageWriter<ShowToast>,
) {
    let address = match parse_address(&request.address) {
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
    // Localhost удалить нельзя: запрос на него просто игнорируется.
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
