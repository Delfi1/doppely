//! Состояния клиента и хуки перехода между ними.
//!
//! У каждого состояния есть пара [`StateHooks::start`] / [`StateHooks::clean`]:
//! `start` выполняется при входе в состояние (`OnEnter`), `clean` — при выходе
//! (`OnExit`). Всё, что состояние создало, освобождается в `clean`, поэтому
//! переход в любое другое состояние не оставляет мусора.

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
    ui::{ConnectionStatusChanged, ScreenChanged, ServersChanged, ShowToast, UiScreen},
};
use crate::servers::{LOCALHOST, ServerList};
use crate::{PlayerAction, renderer::ui_camera};

/// Состояния клиентского приложения.
#[derive(States, Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum ClientState {
    /// Главное меню: выбор сервера, курсор видим, сеть не поднята.
    #[default]
    Menu,
    /// Идёт установка соединения: ждём `Connected` от lightyear.
    Connecting,
    /// Игра идёт: курсор захвачен, HUD поверх сцены.
    InGame,
    /// Esc-меню: ввод игрока отключён, сеть продолжает работать.
    Paused,
    /// Ошибка подключения или потеря связи; можно вернуться в меню.
    Error,
}

impl ClientState {
    pub fn ui_screen(self) -> UiScreen {
        match self {
            Self::Menu => UiScreen::Menu,
            Self::Connecting => UiScreen::Connecting,
            Self::InGame => UiScreen::Game,
            Self::Paused => UiScreen::Paused,
            Self::Error => UiScreen::Error,
        }
    }
}

/// Пара хуков перехода, которую реализует состояние.
pub trait StateHooks {
    /// `OnEnter`: поднять всё, что нужно состоянию.
    fn start(world: &mut World);
    /// `OnExit`: убрать то, что поднял `start` и что состояние накопило.
    fn clean(world: &mut World);
}

/// Помечает сущность состоянием: её удалит `clean` этого состояния.
///
/// Экранные сущности здесь — только то, что живёт в мире (камеры и UI — забота
/// React), но маркер остаётся общим способом привязать временное к состоянию.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct StateEntity(pub ClientState);

/// Выбранный в меню сервер; по умолчанию — localhost.
#[derive(Resource, Debug, Clone, Copy)]
pub struct SelectedServer(pub SocketAddr);

impl Default for SelectedServer {
    fn default() -> Self {
        Self(LOCALHOST)
    }
}

/// Пояснение к экрану ошибки (почему связь не состоялась).
#[derive(Resource, Debug, Default, Clone)]
pub struct ConnectionFailure(pub String);

/// Предупреждение, которое UI покажет, когда сможет: список серверов не
/// прочитался или был повреждён.
#[derive(Resource, Debug, Default, Clone)]
pub struct StartupWarning(pub Option<String>);

pub(super) struct StatesPlugin;

impl Plugin for StatesPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<ClientState>()
            .init_resource::<SelectedServer>()
            .init_resource::<ConnectionFailure>()
            .init_resource::<StartupWarning>();

        // Хуки `start`/`clean` регистрируются на каждое состояние: `OnEnter`
        // берёт состояние из `State<ClientState>`, поэтому пары всегда парные.
        hook_transitions::<ClientState>(app, ClientState::Menu);
        hook_transitions::<ClientState>(app, ClientState::Connecting);
        hook_transitions::<ClientState>(app, ClientState::InGame);
        hook_transitions::<ClientState>(app, ClientState::Paused);
        hook_transitions::<ClientState>(app, ClientState::Error);

        app.add_systems(
            Update,
            (
                sync_screen.run_if(state_changed::<ClientState>),
                (confirm_connection, detect_disconnect).chain(),
                (toggle_pause).run_if(in_play),
                // Простаивающий ввод не должен копиться в действие игрока.
                freeze_input.run_if(not(state_changed::<ClientState>)),
            )
                .chain(),
        );

        // `world.write_message(..)` живёт внутри ECS: до React его доносит именно
        // этот пересыльщик. Ре-рассылка нужна потому, что UI подключается к
        // каналам уже после первого кадра и первые события бы иначе потерялись.
        app.add_systems(
            Update,
            (forward_to_ui, rebroadcast_ui.run_if(every_half_second)).chain(),
        );
    }
}

// ---------------------------------------------------------------------------
// Хуки состояний
// ---------------------------------------------------------------------------

/// Вешает пару `OnEnter`/`OnExit` состояния на его хуки из [`StateHooks`].
fn hook_transitions<S: States + StateHooks + Clone>(app: &mut App, state: S) {
    app.add_systems(OnEnter(state.clone()), |world: &mut World| S::start(world));
    app.add_systems(OnExit(state), |world: &mut World| S::clean(world));
}

impl ClientState {
    fn start(self, world: &mut World) {
        match self {
            Self::Menu => menu_start(world),
            Self::Connecting => connecting_start(world),
            Self::InGame => in_game_start(world),
            Self::Paused | Self::Error => set_cursor(world, false),
        }
    }

    fn clean(self, world: &mut World) {
        match self {
            Self::Menu => despawn_state(world, Self::Menu),
            Self::Connecting => connecting_clean(world),
            Self::InGame => {}
            Self::Paused => set_cursor(world, true),
            Self::Error => {
                world.remove_resource::<ConnectionFailure>();
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
    // Возврат в меню = конец сеанса: сокет и netcode-клиент больше не нужны и
    // не должны пережить следующий сеанс.
    despawn_all::<With<Client>>(world);
    world.spawn(ui_camera());

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
    set_cursor(world, false);

    let server = world.resource::<SelectedServer>().0;
    world.write_message(ConnectionStatusChanged {
        status: "connecting".into(),
        server: server.to_string(),
        message: "Устанавливаем соединение с сервером…".into(),
    });

    spawn_client(world);
}

fn connecting_clean(_world: &mut World) {
    // Соединение здесь НЕ трогаем: `Connecting → InGame` — тот же сеанс, и
    // удалять клиент на выходе из состояния значило бы оборвать связь в момент
    // успешного подключения. Сеанс заканчивается при возврате в меню
    // (`menu_start`) или при ошибке.
}

fn in_game_start(world: &mut World) {
    // Отдельную камеру не создаём: `setup_controller` уже вешает `Camera3d`
    // ребёнком управляемого игрока, как только сервер его реплицирует.
    reset_look(world);
    set_cursor(world, true);
    reset_actions(world);
}

// fn in_game_clean(world: &mut World) {
//     despawn_state(world, ClientState::InGame);
//     set_cursor(world, false);
//     reset_look(world);
//     reset_actions(world);
// }

// ---------------------------------------------------------------------------
// Сеть
// ---------------------------------------------------------------------------

fn spawn_client(world: &mut World) {
    let server = world.resource::<SelectedServer>().0;

    let conditioner = LinkConditionerConfig::average_condition().half();
    let entity = world
        .spawn(ClientSettings {
            server_addr: server,
            conditioner: Some(RecvLinkConditioner::new(conditioner)),
            shared: crate::SHARED_SETTINGS,
        })
        .id();
    world.flush();

    // `ClientSettings::on_add` создаёт `Client`; без него `Connect` ничего не
    // сделает, и подключение зависло бы навсегда.
    if world.get::<Client>(entity).is_some() {
        world.trigger(Connect { entity });
        world.flush();
    } else {
        fail(world, "Не удалось создать сетевой клиент");
    }
}

/// `Connecting` → `InGame`, когда lightyear подтвердил связь.
fn confirm_connection(
    client: Option<Single<(), (With<Client>, With<Connected>)>>,
    state: Res<State<ClientState>>,
    mut commands: Commands,
) {
    if client.is_some() && matches!(state.get(), ClientState::Connecting) {
        commands.set_state(ClientState::InGame);
    }
}

/// Потеря связи → экран ошибки с причиной.
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

// ---------------------------------------------------------------------------
// Пауза
// ---------------------------------------------------------------------------

fn in_play(state: Res<State<ClientState>>) -> bool {
    matches!(state.get(), ClientState::InGame | ClientState::Paused)
}

/// Esc переключает игру и Esc-меню. Кнопки из самого меню обрабатываются
/// наблюдателями в `ui` (`on_pause` / `on_resume`).
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

/// Сбрасывает действия игрока, пока игра не захватила курсор: иначе нажатые
/// в меню клавиши «просочатся» в первый же кадр игры.
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

// ---------------------------------------------------------------------------
// UI
// ---------------------------------------------------------------------------

/// Отправляет React текущий экран.
fn sync_screen(state: Res<State<ClientState>>, events: ReactEvents) {
    events.send(&ScreenChanged {
        screen: state.get().ui_screen().name().into(),
    });
}

/// Пересылает сообщения ECS в React. `ReactEvents` — единственный канал
/// Rust → React, поэтому всё, что пишется в мир, должно пройти здесь.
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

/// Раз в ~0.5 с повторяет экран и список серверов: React подключается к каналам
/// уже после первого кадра, а без повтора мог бы остаться с пустым состоянием.
fn rebroadcast_ui(state: Res<State<ClientState>>, servers: Res<ServerList>, events: ReactEvents) {
    events.send(&ScreenChanged {
        screen: state.get().ui_screen().name().into(),
    });
    events.send(&ServersChanged {
        servers: servers.all().iter().map(Into::into).collect(),
    });
}

fn every_half_second(time: Res<Time>) -> bool {
    // Делим накопленное время на 0.5 с, чтобы не заводить свой счётчик: так
    // условие не зависит от частоты кадров.
    (time.elapsed_secs() * 2.0) as u64 != ((time.elapsed_secs() - time.delta_secs()) * 2.0) as u64
}

// ---------------------------------------------------------------------------
// Утилиты
// ---------------------------------------------------------------------------

/// Убирает сущности, помеченные [`StateEntity`] состояния.
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

/// Захват курсора игрой: в игре он скрыт и ограничен окном, иначе — свободен.
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
