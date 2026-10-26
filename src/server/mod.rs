use bevy::ecs::{lifecycle::HookContext, world::DeferredWorld};
use bevy::state::state::States;
use doppely::config::ServerConfig;
use doppely::scene::{SceneComponent, SceneObject};
use doppely::status::ServerStatusReply;
use doppely::*;
use leafwing_input_manager::action_state::ActionState;
use lightyear::{
    connection::{client::Disconnecting, client_of::ClientOf, server::Start},
    prelude::{server::*, *},
};
use std::collections::VecDeque;
use std::io::BufRead;

mod editor;
mod online;
mod train;
use editor::SceneCounter;
use online::ServerStatus;
use train::*;

const SEND_INTERVAL: Duration = Duration::from_millis(16);
const DEFAULT_PLAYER_SPAWN: Vec3 = Vec3::new(0.0, 2.0, 4.0);

#[derive(States, Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum ServerState {
    #[default]
    Lobby,
    Game,
    Edit,
}

#[derive(Message, Debug, Clone)]
pub struct ConsoleCommand(pub Vec<String>);

pub struct ServerPlugin;

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<ServerState>();
        app.init_resource::<SceneCounter>();
        app.init_resource::<CurrentScene>();
        app.init_resource::<EditorHistory>();
        app.init_resource::<PendingActions>();
        app.add_message::<ConsoleCommand>();
        app.add_plugins(TrainPlugin);

        app.add_observer(handle_new);
        app.add_observer(reject_unwelcome);

        app.add_systems(Startup, (setup, spawn_lobby, start));
        app.add_systems(PreUpdate, process_input);
        app.add_systems(
            Update,
            (
                handle_commands,
                handle_editor_requests,
                refresh_editor_info,
                sync_lobby,
                sync_mode,
                handle_start_requests,
                despawn_orphans,
                publish_status,
            ),
        );
        app.add_systems(OnEnter(ServerState::Game), spawn_players);
        app.add_systems(OnExit(ServerState::Game), despawn_players);
        app.add_systems(OnEnter(ServerState::Edit), pause_physics);
        app.add_systems(OnExit(ServerState::Edit), resume_physics);
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
            let bind = world
                .get_resource::<ServerConfig>()
                .map(ServerConfig::bind_addr)
                .unwrap_or_else(|| ServerConfig::default().bind_addr());

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
            entity_mut.insert((LocalAddr(bind), ServerUdpIo::default()));
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

    let (config, config_path, warning) = ServerConfig::read();
    bevy::log::info!("Конфиг: {}", config_path.display());
    if let Some(warning) = &warning {
        bevy::log::warn!("{warning}");
    }

    let conditioner = LinkConditionerConfig::average_condition().half();
    let shared = config.shared();
    let bind = config.bind_addr();
    app.insert_resource(config);
    app.world_mut().spawn(ServerSettings {
        conditioner: Some(RecvLinkConditioner::new(conditioner.clone())),
        shared,
    });

    let status = ServerStatus::default();
    online::spawn(ServerStatusReply::addr_for(bind), status.clone());
    app.insert_resource(status);

    app.add_plugins(ServerPlugin);
    app.init_resource::<ReadLine>();
    app.insert_resource(StartupScene(scene_requested()));

    if editing_requested() {
        bevy::log::info!("Режим редактирования (--editing): команды сцены включены");
        app.world_mut()
            .resource_mut::<NextState<ServerState>>()
            .set(ServerState::Edit);
    }

    app.run();
}

#[derive(Resource, Default, Debug)]
struct StartupScene(Option<String>);

#[derive(Resource, Default, Debug)]
struct CurrentScene(Option<String>);

fn editing_requested() -> bool {
    std::env::args().any(|arg| arg == "--editing")
}

fn scene_requested() -> Option<String> {
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--scene" {
            return args.next();
        }
        if let Some(name) = arg.strip_prefix("--scene=") {
            return Some(name.to_string());
        }
    }
    None
}

fn setup(
    mut commands: Commands,
    mut counter: ResMut<SceneCounter>,
    startup: Res<StartupScene>,
    mut current: ResMut<CurrentScene>,
) {
    let objects = match &startup.0 {
        Some(name) => match doppely::scene::load(name) {
            Ok(objects) => {
                bevy::log::info!("Сцена «{name}»: {} объектов", objects.len());
                current.0 = Some(name.clone());
                objects
            }
            Err(error) => {
                bevy::log::warn!("{error}; создаётся пустой мир с кубом");
                doppely::scene::empty_scene()
            }
        },
        None => {
            bevy::log::info!("Сцена не задана: пустой мир с кубом");
            doppely::scene::empty_scene()
        }
    };

    editor::spawn_scene(&mut commands, &mut counter, objects);
}

fn start(mut commands: Commands, server: Single<Entity, With<Server>>) {
    commands.trigger(Start {
        entity: server.into_inner(),
    });
}

fn handle_new(trigger: On<Add<LinkOf>>, mut commands: Commands) {
    commands.entity(trigger.entity).insert(ReplicationSender);
}

fn spawn_lobby(mut commands: Commands) {
    commands.spawn((
        Name::new("Lobby"),
        LobbyInfo {
            players: 0,
            max_players: MAX_PLAYERS,
        },
        EditorInfo::default(),
        ServerMode::Lobby,
        Replicate::to_clients(NetworkTarget::All),
    ));
}

fn sync_lobby(
    mut lobby: Query<&mut LobbyInfo>,
    clients: Query<(), (With<ClientOf>, With<Connected>)>,
) {
    let players = clients.iter().count() as u8;
    for mut info in &mut lobby {
        if info.players != players {
            info.players = players;
        }
    }
}

fn pause_physics(mut time: ResMut<Time<Physics>>) {
    time.pause();
}

fn resume_physics(mut time: ResMut<Time<Physics>>) {
    time.unpause();
}

fn sync_mode(state: Res<State<ServerState>>, mut lobby: Query<&mut ServerMode>) {
    if !state.is_changed() {
        return;
    }
    let mode = match state.get() {
        ServerState::Lobby => ServerMode::Lobby,
        ServerState::Game => ServerMode::Game,
        ServerState::Edit => ServerMode::Edit,
    };
    for mut current in &mut lobby {
        if *current != mode {
            *current = mode;
        }
    }
}

fn handle_start_requests(
    mut receivers: Query<&mut MessageReceiver<StartGame>>,
    state: Res<State<ServerState>>,
    mut next: ResMut<NextState<ServerState>>,
) {
    for mut receiver in &mut receivers {
        for _ in receiver.receive() {
            if *state.get() == ServerState::Lobby {
                bevy::log::info!("Starting the game");
                next.set(ServerState::Game);
            }
        }
    }
}

fn handle_editor_requests(
    mut receivers: Query<&mut MessageReceiver<EditorCommand>>,
    state: Res<State<ServerState>>,
    mut commands: Commands,
    mut counter: ResMut<SceneCounter>,
    mut scene: Query<(Entity, &mut SceneObject, &mut Position)>,
    mut info: Query<&mut EditorInfo>,
    mut current: ResMut<CurrentScene>,
    mut history: ResMut<EditorHistory>,
    mut pending: ResMut<PendingActions>,
    time: Res<Time>,
) {
    let mut queue: VecDeque<EditorAction> = std::mem::take(&mut pending.0);
    for mut receiver in &mut receivers {
        for command in receiver.receive() {
            queue.push_back(command.action.clone());
        }
    }

    let mut restored = false;

    while let Some(action) = queue.pop_front() {
        if restored {
            queue.push_front(action);
            break;
        }

        if *state.get() != ServerState::Edit {
            set_editor_result(&mut info, &Err("Сервер не в режиме редактирования".into()));
            continue;
        }

        let now = time.elapsed_secs();
        let structural = matches!(
            action,
            EditorAction::Load { .. } | EditorAction::Undo | EditorAction::Redo
        );
        let result = match action {
            EditorAction::Save { name } => {
                let outcome = run_scene_words(
                    &["save".to_string(), name.clone()],
                    &mut commands,
                    &mut counter,
                    &mut scene,
                );
                if outcome.is_ok() {
                    current.0 = Some(name);
                }
                outcome
            }
            EditorAction::Load { name } => {
                record_history(&mut history, &scene, now);
                let outcome = run_scene_words(
                    &["load".to_string(), name.clone()],
                    &mut commands,
                    &mut counter,
                    &mut scene,
                );
                if outcome.is_ok() {
                    current.0 = Some(name);
                }
                outcome
            }
            EditorAction::Refresh => run_scene_words(
                &["scenes".to_string()],
                &mut commands,
                &mut counter,
                &mut scene,
            ),
            EditorAction::Spawn { kind, position } => {
                record_history(&mut history, &scene, now);
                spawn_from_client(&mut counter, &mut commands, kind, position)
            }
            EditorAction::Move { id, position } => {
                match record_and_find(&mut history, &mut scene, id, now) {
                    Some((_, mut object, mut pos)) => {
                        object.position = position;
                        pos.0 = position;
                        Ok(String::new())
                    }
                    None => Err(format!("Объект #{id} не найден")),
                }
            }
            EditorAction::SetTransform {
                id,
                position,
                rotation,
                size,
            } => match record_and_find(&mut history, &mut scene, id, now) {
                Some((entity, mut object, mut pos)) => {
                    object.position = position;
                    object.rotation = rotation;
                    object.size = size;
                    pos.0 = position;
                    commands
                        .entity(entity)
                        .insert((Rotation(object.quat()), ObjectSize(size)));
                    Ok(String::new())
                }
                None => Err(format!("Объект #{id} не найден")),
            },
            EditorAction::Remove { id } => {
                match record_and_find(&mut history, &mut scene, id, now) {
                    Some((entity, _, _)) => {
                        commands.entity(entity).despawn();
                        Ok(format!("Удалён объект #{id}"))
                    }
                    None => Err(format!("Объект #{id} не найден")),
                }
            }
            EditorAction::RemoveMany { ids } => {
                if ids.is_empty() {
                    Err("Ничего не выбрано".into())
                } else {
                    record_history(&mut history, &scene, now);
                    let mut removed = 0;
                    for (entity, object, _) in scene.iter() {
                        if ids.contains(&object.id) {
                            commands.entity(entity).despawn();
                            removed += 1;
                        }
                    }
                    if removed == 0 {
                        Err("Объекты не найдены".into())
                    } else {
                        Ok(format!("Удалено объектов: {removed}"))
                    }
                }
            }
            EditorAction::SetComponents { id, components } => {
                match record_and_find(&mut history, &mut scene, id, now) {
                    Some((entity, mut object, _)) => {
                        object.components = components;
                        let updated = object.clone();
                        commands.entity(entity).despawn();
                        editor::spawn_object(&mut commands, &updated);
                        Ok(format!("Обновлён объект #{id}"))
                    }
                    None => Err(format!("Объект #{id} не найден")),
                }
            }
            EditorAction::Undo => {
                let current = take_snapshot(&scene);
                match history.undo.pop() {
                    Some(previous) => {
                        history.redo.push(current);
                        history.reset_coalesce();
                        restore_scene(previous, &mut commands, &mut counter, &scene);
                        Ok("Отменено".into())
                    }
                    None => Err("Нечего отменять".into()),
                }
            }
            EditorAction::Redo => {
                let current = take_snapshot(&scene);
                match history.redo.pop() {
                    Some(next) => {
                        history.undo.push(current);
                        history.reset_coalesce();
                        restore_scene(next, &mut commands, &mut counter, &scene);
                        Ok("Повторено".into())
                    }
                    None => Err("Нечего повторять".into()),
                }
            }
        };

        match &result {
            Ok(message) if !message.is_empty() => bevy::log::info!("[client] {message}"),
            Ok(_) => {}
            Err(error) => bevy::log::warn!("[client] {error}"),
        }
        set_editor_result(&mut info, &result);

        if structural {
            restored = true;
        }
    }

    pending.0 = queue;
}

#[derive(Resource, Default, Debug)]
struct PendingActions(VecDeque<EditorAction>);

#[derive(Resource, Debug)]
struct EditorHistory {
    undo: Vec<Vec<SceneObject>>,
    redo: Vec<Vec<SceneObject>>,
    last: f32,
}

impl Default for EditorHistory {
    fn default() -> Self {
        Self {
            undo: Vec::new(),
            redo: Vec::new(),
            last: f32::NEG_INFINITY,
        }
    }
}

const HISTORY_LIMIT: usize = 64;
const HISTORY_COALESCE: f32 = 0.7;

impl EditorHistory {
    fn record(&mut self, snapshot: Vec<SceneObject>, now: f32, coalesce: bool) {
        if coalesce && now - self.last < HISTORY_COALESCE {
            self.last = now;
            return;
        }

        self.last = now;
        self.undo.push(snapshot);
        if self.undo.len() > HISTORY_LIMIT {
            self.undo.remove(0);
        }
        self.redo.clear();
    }

    fn reset_coalesce(&mut self) {
        self.last = f32::NEG_INFINITY;
    }
}

fn take_snapshot(scene: &Query<(Entity, &mut SceneObject, &mut Position)>) -> Vec<SceneObject> {
    let mut objects: Vec<SceneObject> = scene.iter().map(|(_, object, _)| object.clone()).collect();
    objects.sort_by_key(|object| object.id);
    objects
}

fn record_history(
    history: &mut EditorHistory,
    scene: &Query<(Entity, &mut SceneObject, &mut Position)>,
    now: f32,
) {
    history.record(take_snapshot(scene), now, true);
}

fn record_and_find<'a>(
    history: &mut EditorHistory,
    scene: &'a mut Query<(Entity, &mut SceneObject, &mut Position)>,
    id: u32,
    now: f32,
) -> Option<(Entity, Mut<'a, SceneObject>, Mut<'a, Position>)> {
    if !scene.iter().any(|(_, object, _)| object.id == id) {
        return None;
    }

    record_history(history, &*scene, now);
    scene.iter_mut().find(|(_, object, _)| object.id == id)
}

fn restore_scene(
    objects: Vec<SceneObject>,
    commands: &mut Commands,
    counter: &mut SceneCounter,
    scene: &Query<(Entity, &mut SceneObject, &mut Position)>,
) {
    for (entity, _, _) in scene.iter() {
        commands.entity(entity).despawn();
    }

    let next = objects
        .iter()
        .map(|object| object.id + 1)
        .max()
        .unwrap_or(0);
    for object in &objects {
        editor::spawn_object(commands, object);
    }
    counter.reset_to(next);
}

fn spawn_from_client(
    counter: &mut SceneCounter,
    commands: &mut Commands,
    kind: ObjectType,
    position: Vec3,
) -> Result<String, String> {
    let id = counter.next();
    let size = doppely::scene::default_size(kind);
    let position = Vec3::new(position.x, doppely::scene::ground_y(kind, size), position.z);

    let object = SceneObject::new(id, kind, size, position, &format!("{kind:?} {id}"))
        .with(SceneComponent::Friction { coefficient: 0.8 });

    editor::spawn_object(commands, &object);
    Ok(format!("Создан объект #{id} ({kind:?})"))
}

fn run_scene_words(
    words: &[String],
    commands: &mut Commands,
    counter: &mut SceneCounter,
    scene: &mut Query<(Entity, &mut SceneObject, &mut Position)>,
) -> Result<String, String> {
    handle_scene_command(words, commands, counter, scene)
}

fn refresh_editor_info(
    mut info: Query<&mut EditorInfo>,
    current: Res<CurrentScene>,
    state: Res<State<ServerState>>,
    history: Res<EditorHistory>,
    mut tick: Local<u32>,
) {
    *tick = tick.wrapping_add(1);
    let periodic = tick.is_multiple_of(64);
    if !periodic && !state.is_changed() && !current.is_changed() && !history.is_changed() {
        return;
    }

    let scenes = doppely::scene::scenes();
    let scene = current.0.clone().unwrap_or_default();
    let can_undo = !history.undo.is_empty();
    let can_redo = !history.redo.is_empty();

    for mut editor in &mut info {
        if editor.scenes != scenes {
            editor.scenes = scenes.clone();
        }
        if editor.scene != scene {
            editor.scene = scene.clone();
        }
        if editor.can_undo != can_undo {
            editor.can_undo = can_undo;
        }
        if editor.can_redo != can_redo {
            editor.can_redo = can_redo;
        }
    }
}

fn set_editor_result(info: &mut Query<&mut EditorInfo>, result: &Result<String, String>) {
    for mut editor in info.iter_mut() {
        match result {
            Ok(message) => {
                editor.ok = true;
                editor.message = message.clone();
            }
            Err(error) => {
                editor.ok = false;
                editor.message = error.clone();
            }
        }
    }
}

fn reject_unwelcome(
    trigger: On<Add<Connected>>,
    state: Res<State<ServerState>>,
    clients: Query<(), (With<ClientOf>, With<Connected>)>,
    mut commands: Commands,
) {
    let accepting = matches!(state.get(), ServerState::Lobby | ServerState::Edit);
    let room = clients.iter().count() <= MAX_PLAYERS as usize;
    if !accepting || !room {
        bevy::log::info!(
            "Rejecting client {:?}: server is not accepting",
            trigger.entity
        );
        commands.entity(trigger.entity).insert(Disconnecting);
    }
}

fn spawn_players(
    mut commands: Commands,
    clients: Query<(Entity, &RemoteId), (With<ClientOf>, With<Connected>)>,
    spawnpoints: Query<&Position, With<doppely::scene::PlayerSpawnpoint>>,
) {
    let mut points: Vec<Vec3> = spawnpoints.iter().map(|position| position.0).collect();
    points.sort_by(|a, b| {
        a.x.total_cmp(&b.x)
            .then(a.z.total_cmp(&b.z))
            .then(a.y.total_cmp(&b.y))
    });

    if points.is_empty() {
        bevy::log::error!(
            "В сцене нет ни одной точки спавна (PlayerSpawnpoint) — игроки появятся в {DEFAULT_PLAYER_SPAWN}"
        );
    }

    for (index, (link, &RemoteId(client_id))) in clients.iter().enumerate() {
        let position = points
            .get(index % points.len().max(1))
            .copied()
            .unwrap_or(DEFAULT_PLAYER_SPAWN);

        commands.spawn((
            RigidBody::Dynamic,
            ObjectType::Capsule,
            LockedAxes::new()
                .lock_rotation_x()
                .lock_rotation_y()
                .lock_rotation_z(),
            ObjectSize(Vec2::new(0.5, 1.2).xyx()),
            Position(position),
            Friction::new(0.95),
            Name::new("Player"),
            ActionState::<PlayerAction>::default(),
            PlayerId(client_id),
            DisableReplicateHierarchy,
            Replicate::to_clients(NetworkTarget::All),
            PredictionTarget::to_clients(NetworkTarget::All),
            InterpolationTarget::to_clients(NetworkTarget::AllExceptSingle(client_id)),
            ControlledBy {
                owner: link,
                lifetime: Default::default(),
            },
        ));
    }
}

fn despawn_players(mut commands: Commands, players: Query<Entity, With<PlayerId>>) {
    for entity in &players {
        commands.entity(entity).despawn();
    }
}

fn publish_status(
    status: Res<ServerStatus>,
    state: Res<State<ServerState>>,
    clients: Query<(), (With<ClientOf>, With<Connected>)>,
) {
    let mode = match state.get() {
        ServerState::Lobby => ServerMode::Lobby,
        ServerState::Game => ServerMode::Game,
        ServerState::Edit => ServerMode::Edit,
    };
    status.set(clients.iter().count() as u8, mode);
}

fn despawn_orphans(
    mut commands: Commands,
    players: Query<(Entity, &ControlledBy)>,
    owners: Query<(), With<ClientOf>>,
) {
    for (entity, controlled) in &players {
        if owners.get(controlled.owner).is_err() {
            commands.entity(entity).despawn();
        }
    }
}

#[derive(Resource, Default)]
pub struct ReadLine {
    task: Option<std::thread::JoinHandle<Option<String>>>,
    closed: bool,
}

fn process_input(mut read: ResMut<ReadLine>, mut out: MessageWriter<ConsoleCommand>) {
    if read.closed {
        return;
    }

    if read.task.is_none() {
        read.task = Some(std::thread::spawn(|| {
            let mut line = String::new();
            match std::io::stdin().lock().read_line(&mut line) {
                Ok(0) | Err(_) => None,
                Ok(_) => Some(line.trim().to_lowercase()),
            }
        }));
    }

    let Some(task) = read.task.take() else {
        return;
    };
    if !task.is_finished() {
        read.task = Some(task);
        return;
    }

    match task.join().unwrap_or(None) {
        Some(line) => {
            let words: Vec<String> = line.split_whitespace().map(str::to_string).collect();
            if !words.is_empty() {
                out.write(ConsoleCommand(words));
            }
        }
        None => {
            bevy::log::info!("Консоль закрыта, ввод команд остановлен");
            read.closed = true;
        }
    }
}

fn help_text() -> &'static str {
    "\
команды: players, debug, lobby, game, edit, exit, help\n\
сцена (только в режиме edit):\n\
  list — объекты сцены\n\
  spawn box|sphere|capsule|cylinder|cone|spawnpoint|crate <x> <y> <z> [размеры]\n\
  room <x> <z> [ширина глубина] [высота]\n\
  remove <id>|all\n\
  undo / redo — отменить или повторить правку\n\
  scenes — сохранённые сцены (assets/scenes)\n\
  save <имя> / load <имя>"
}

fn numbers(words: &[String], from: usize) -> Vec<f32> {
    words
        .iter()
        .skip(from)
        .filter_map(|word| word.parse::<f32>().ok())
        .collect()
}

#[allow(clippy::too_many_arguments)]
fn handle_commands(
    mut lines: MessageReader<ConsoleCommand>,
    mut commands: Commands,
    state: Res<State<ServerState>>,
    mut next: ResMut<NextState<ServerState>>,
    mut counter: ResMut<SceneCounter>,
    mut scene: Query<(Entity, &mut SceneObject, &mut Position)>,
    players: Query<Entity, With<PlayerId>>,
    mut current: ResMut<CurrentScene>,
    mut history: ResMut<EditorHistory>,
    time: Res<Time>,
) {
    let editing = *state.get() == ServerState::Edit;

    for ConsoleCommand(words) in lines.read() {
        let command = words.first().map(String::as_str).unwrap_or_default();

        match command {
            "exit" | "quit" | "stop" => {
                bevy::log::info!("Stopping server...");
                std::process::exit(0);
            }
            "help" => bevy::log::info!("\n{}", help_text()),
            "players" => bevy::log::info!("Players: {}", players.iter().count()),
            "lobby" => next.set(ServerState::Lobby),
            "game" => next.set(ServerState::Game),
            "edit" => next.set(ServerState::Edit),
            "undo" | "redo" => {
                if !editing {
                    bevy::log::warn!("Команды сцены работают только в режиме edit");
                    continue;
                }
                let current_scene = take_snapshot(&scene);
                let restored = if command == "undo" {
                    history.undo.pop().map(|previous| {
                        history.redo.push(current_scene);
                        previous
                    })
                } else {
                    history.redo.pop().map(|next| {
                        history.undo.push(current_scene);
                        next
                    })
                };

                match restored {
                    Some(objects) => {
                        history.reset_coalesce();
                        let count = objects.len();
                        restore_scene(objects, &mut commands, &mut counter, &mut scene);
                        bevy::log::info!(
                            "{}: {} объектов",
                            if command == "undo" {
                                "Отменено"
                            } else {
                                "Повторено"
                            },
                            count
                        );
                    }
                    None => bevy::log::warn!(
                        "{}",
                        if command == "undo" {
                            "Нечего отменять"
                        } else {
                            "Нечего повторять"
                        }
                    ),
                }
            }
            "debug" => {
                for (_, object, position) in scene.iter_mut().take(32) {
                    bevy::log::info!("{:?} Position: {:.3}", object.kind, position.0);
                }
            }
            "list" | "spawn" | "room" | "remove" | "save" | "load" | "scenes" => {
                if !editing {
                    bevy::log::warn!("Команды сцены работают только в режиме edit");
                    continue;
                }
                if matches!(command, "spawn" | "room" | "remove" | "load") {
                    record_history(&mut history, &scene, time.elapsed_secs());
                }
                match handle_scene_command(&words, &mut commands, &mut counter, &mut scene) {
                    Ok(message) => {
                        bevy::log::info!("{message}");
                        if matches!(command, "save" | "load") {
                            if let Some(name) = words.get(1) {
                                current.0 = Some(name.clone());
                            }
                        }
                    }
                    Err(error) => bevy::log::warn!("{error}"),
                }
            }
            other => bevy::log::warn!("Unknown command: \"{other}\""),
        }
    }
}

fn handle_scene_command(
    words: &[String],
    commands: &mut Commands,
    counter: &mut SceneCounter,
    scene: &mut Query<(Entity, &mut SceneObject, &mut Position)>,
) -> Result<String, String> {
    match words.first().map(String::as_str).unwrap_or_default() {
        "list" => {
            let mut items: Vec<&SceneObject> = scene.iter().map(|(_, object, _)| object).collect();
            items.sort_by_key(|object| object.id);
            if items.is_empty() {
                bevy::log::info!("Сцена пуста");
            }
            for object in items {
                bevy::log::info!(
                    "#{:<3} {:<12} pos ({:.1}, {:.1}, {:.1}) size ({:.1}, {:.1}, {:.1}){} [{:?}]",
                    object.id,
                    object.name,
                    object.position.x,
                    object.position.y,
                    object.position.z,
                    object.size.x,
                    object.size.y,
                    object.size.z,
                    if object.dynamic() { " dynamic" } else { "" },
                    object.components,
                );
            }
            Ok(String::new())
        }
        "spawn" => {
            let name = words.get(1).map(String::as_str).unwrap_or("box");
            let values = numbers(words, 2);
            if values.len() < 3 {
                return Err(
                    "Использование: spawn box|sphere|capsule|cylinder|cone|spawnpoint|crate <x> <y> <z> [размеры]"
                        .into(),
                );
            }
            let position = Vec3::new(values[0], values[1], values[2]);

            let kind = match name {
                "sphere" => ObjectType::Sphere,
                "capsule" => ObjectType::Capsule,
                "cylinder" => ObjectType::Cylinder,
                "cone" => ObjectType::Cone,
                "spawnpoint" => ObjectType::Spawnpoint,
                _ => ObjectType::Cuboid,
            };

            let size = match values.len() {
                4 => Vec3::splat(values[3]),
                5 => Vec3::new(values[3], values[4], values[3]),
                count if count >= 6 => Vec3::new(values[3], values[4], values[5]),
                _ if name == "crate" => Vec3::splat(0.5),
                _ => doppely::scene::default_size(kind),
            };

            let id = counter.next();
            let mut object = SceneObject::new(id, kind, size, position, &format!("{name} {id}"))
                .with(SceneComponent::Friction { coefficient: 0.8 });

            if name == "crate" {
                object = object
                    .with(SceneComponent::Dynamic)
                    .with(SceneComponent::Glow)
                    .with(SceneComponent::Light {
                        intensity: 100_000.0,
                        radius: 10.0,
                    });
            }

            editor::spawn_object(commands, &object);
            Ok(format!("Создан объект #{id}"))
        }
        "room" => {
            let values = numbers(words, 1);
            let x = values.first().copied().unwrap_or(0.0);
            let z = values.get(1).copied().unwrap_or(0.0);
            let width = values.get(2).copied().unwrap_or(10.0);
            let depth = values.get(3).copied().unwrap_or(10.0);
            let height = values.get(4).copied().unwrap_or(3.0);

            for mut object in editor::room(x, z, width, depth, height) {
                object.id = counter.next();
                editor::spawn_object(commands, &object);
            }
            Ok(format!(
                "Комната {width}x{depth}, высота {height}, центр ({x}, {z})"
            ))
        }
        "remove" => match words.get(1).map(String::as_str) {
            Some("all") => {
                for (entity, _, _) in scene.iter() {
                    commands.entity(entity).despawn();
                }
                Ok("Сцена очищена".into())
            }
            Some(raw) => match raw.parse::<u32>() {
                Ok(id) => {
                    let mut removed = false;
                    for (entity, object, _) in scene.iter() {
                        if object.id == id {
                            commands.entity(entity).despawn();
                            removed = true;
                        }
                    }
                    if removed {
                        Ok(format!("Удалён объект #{id}"))
                    } else {
                        Err(format!("Объект #{id} не найден"))
                    }
                }
                Err(_) => Err("Использование: remove <id>|all".into()),
            },
            None => Err("Использование: remove <id>|all".into()),
        },
        "save" => {
            let Some(name) = words.get(1) else {
                return Err(format!(
                    "Использование: save <имя> (сохранение в {})",
                    doppely::scene::dir().display()
                ));
            };
            let scene_objects: Vec<SceneObject> =
                scene.iter().map(|(_, object, _)| object.clone()).collect();
            doppely::scene::save(name, &scene_objects).map(|path| {
                format!(
                    "Сцена «{name}» сохранена: {} ({} объектов)",
                    path.display(),
                    scene_objects.len()
                )
            })
        }
        "load" => {
            let Some(name) = words.get(1) else {
                return Err("Использование: load <имя>".into());
            };
            let objects = doppely::scene::load(name)?;

            for (entity, _, _) in scene.iter() {
                commands.entity(entity).despawn();
            }
            let mut count = 0;
            for mut object in objects {
                object.id = count;
                editor::spawn_object(commands, &object);
                count += 1;
            }
            counter.reset_to(count);
            Ok(format!("Сцена «{name}» загружена: {count} объектов"))
        }
        "scenes" => {
            let names = doppely::scene::scenes();
            if names.is_empty() {
                bevy::log::info!(
                    "Сцен нет. Каталог: {} (создаётся командой save <имя>)",
                    doppely::scene::dir().display()
                );
            } else {
                bevy::log::info!("Сцены в {}:", doppely::scene::dir().display());
                for name in names {
                    bevy::log::info!("  {name}");
                }
            }
            Ok(String::new())
        }
        other => Err(format!("Unknown command: \"{other}\"")),
    }
}
