//! Список серверов клиента.
//!

use std::{
    fmt,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::{Path, PathBuf},
    str::FromStr,
    time::Duration,
};

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::SERVER_PORT;

/// Имя файла со списком серверов (внутри `assets/`).
pub const SERVERS_FILE: &str = "servers.json";

/// Адрес локального сервера. Сюда же биндится сервер, поэтому клиент ходит
/// именно на `127.0.0.1`.
pub const LOCALHOST: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), SERVER_PORT);

/// Локальный сервер: всегда первый и неудаляемый.
pub const LOCAL_HOST_ID: u32 = 0;

/// Сервер, сохранённый в `assets/servers.json`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct ServerEntry {
    pub name: String,
    pub address: String,
}

/// Сервер в том виде, в котором его показывает UI. Отдельный тип, потому что
/// localhost существует только в рантайме и не совпадает с файлом один к одному
/// (у него нет записи в `servers.json`, а `id` считаются по всему списку).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Server {
    pub id: u32,
    pub name: String,
    pub address: String,
    pub local: bool,
}

impl Server {
    pub fn localhost() -> Self {
        Self {
            id: LOCAL_HOST_ID,
            name: "Localhost".to_string(),
            address: LOCALHOST.to_string(),
            local: true,
        }
    }

    /// Адрес из списка — всегда валидный `SocketAddr` (см. [`parse_address`]).
    pub fn socket_addr(&self) -> SocketAddr {
        self.address
            .parse()
            .expect("адреса в списке серверов валидируются при загрузке и добавлении")
    }

    /// Файловая запись этого сервера; у localhost её нет.
    pub fn entry(&self) -> Option<ServerEntry> {
        (!self.local).then(|| ServerEntry {
            name: self.name.clone(),
            address: self.address.clone(),
        })
    }
}

/// Текущий список серверов: localhost + содержимое `servers.json`.
#[derive(Debug, Clone, Default, Resource)]
pub struct ServerList {
    entries: Vec<ServerEntry>,
}

impl ServerList {
    pub fn with_servers(entries: Vec<ServerEntry>) -> Self {
        Self { entries }
    }

    /// Весь список для UI, localhost первым.
    pub fn all(&self) -> Vec<Server> {
        let mut servers = Vec::with_capacity(self.entries.len() + 1);
        servers.push(Server::localhost());
        servers.extend(
            self.entries
                .iter()
                .enumerate()
                .map(|(index, entry)| Server {
                    id: index as u32 + 1,
                    name: entry.name.clone(),
                    address: entry.address.clone(),
                    local: false,
                }),
        );
        servers
    }

    pub fn get(&self, id: u32) -> Option<Server> {
        self.all().into_iter().find(|server| server.id == id)
    }

    /// Добавляет сервер в конец списка. Адрес уже должен быть нормализован
    /// [`parse_address`]-ом; повторный адрес (включая localhost) не добавляется.
    pub fn insert(&mut self, name: String, address: String) -> bool {
        if address == LOCALHOST.to_string()
            || self.entries.iter().any(|entry| entry.address == address)
        {
            return false;
        }
        self.entries.push(ServerEntry { name, address });
        true
    }

    /// Удаляет добавленный сервер. Localhost удалить нельзя.
    pub fn remove(&mut self, id: u32) -> bool {
        let Some(index) = id.checked_sub(1).map(|id| id as usize) else {
            return false;
        };
        if index >= self.entries.len() {
            return false;
        }
        self.entries.remove(index);
        true
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Приводит введённый адрес к `host:port` с портом сервера по умолчанию.
///
/// Принимает `127.0.0.1`, `127.0.0.1:5000`, `192.168.3.5`, `[::1]:5000` и
/// `localhost`. Схема `udp://` и пробелы отбрасываются — игрок обычно копирует
/// адрес из браузера или чата.
pub fn parse_address(input: &str) -> Result<String, String> {
    let raw = input.trim();
    let raw = raw
        .strip_prefix("udp://")
        .or_else(|| raw.strip_prefix("UDP://"))
        .unwrap_or(raw)
        .trim()
        .trim_end_matches('/');
    if raw.is_empty() {
        return Err("Введите адрес сервера".to_string());
    }

    if let Ok(addr) = SocketAddr::from_str(raw) {
        return Ok(addr.to_string());
    }
    if let Ok(ip) = IpAddr::from_str(raw) {
        return Ok(SocketAddr::new(ip, SERVER_PORT).to_string());
    }
    if raw.eq_ignore_ascii_case("localhost") {
        return Ok(LOCALHOST.to_string());
    }
    Err(format!("Не удалось разобрать адрес «{raw}»"))
}

/// Где живёт `servers.json`.
///
/// В dev-запуске (`cargo run`) файл лежит ровно там, где его ждёт
/// [`AssetPlugin`](bevy::asset::AssetPlugin) — `assets/` в корне проекта, а при
/// отсутствии файла он создаётся. В собранной игре `assets/` может быть только
/// для чтения (Program Files), поэтому запись уходит в конфиг-каталог ОС, а
/// `assets/servers.json` из поставки используется как начальные данные.
#[derive(Debug, Clone, Resource)]
pub struct ServersPath {
    read: PathBuf,
    write: PathBuf,
}

impl ServersPath {
    pub fn resolve() -> Self {
        let read = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("assets")
            .join(SERVERS_FILE);

        if cfg!(debug_assertions) || !read.exists() {
            return Self {
                write: read.clone(),
                read,
            };
        }

        let write = config_dir()
            .map(|dir| dir.join(SERVERS_FILE))
            .unwrap_or_else(|| read.clone());
        Self { read, write }
    }

    pub fn read_path(&self) -> &Path {
        &self.read
    }

    pub fn write_path(&self) -> &Path {
        &self.write
    }

    /// Читает список серверов. Отсутствующий файл — пустой список, битый файл
    /// не роняет клиент: он откладывается в `servers.json.bak`, а список
    /// начинается заново с одного localhost.
    pub fn load(&self) -> (Vec<ServerEntry>, Option<String>) {
        let data = match std::fs::read_to_string(&self.read) {
            Ok(data) => data,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                if let Err(error) = self.save(&[]) {
                    bevy::log::warn!("не удалось создать {:?}: {error}", self.write);
                }
                return (Vec::new(), None);
            }
            Err(error) => {
                return (
                    Vec::new(),
                    Some(format!("Не удалось прочитать {:?}: {error}", self.read)),
                );
            }
        };

        let result: Result<Vec<ServerEntry>, _> = serde_json::from_str(&data);
        match result {
            Ok(entries) => {
                let mut seen = vec![LOCALHOST.to_string()];
                let entries: Vec<_> = entries
                    .into_iter()
                    .filter(|entry| {
                        let addr = parse_address(&entry.address).unwrap_or_default();
                        if addr.is_empty() || seen.contains(&addr) {
                            return false;
                        }
                        seen.push(addr);
                        true
                    })
                    .collect();
                (entries, None)
            }
            Err(error) => {
                let backup = self.read.with_extension("json.bak");
                let _ = std::fs::rename(&self.read, &backup);
                let _ = self.save(&[]);
                bevy::log::warn!("{error}, файл отложен в {:?}", backup);
                (
                    Vec::new(),
                    Some(format!("Список серверов повреждён, начат заново ({error})")),
                )
            }
        }
    }

    /// Атомарная запись: сначала во временный файл, затем переименование, чтобы
    /// падение по ходу записи не оставило обрезанный список.
    pub fn save(&self, entries: &[ServerEntry]) -> std::io::Result<()> {
        if let Some(parent) = self.write.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(entries)
            .map_err(|error| std::io::Error::other(error.to_string()))?;
        let tmp = self.write.with_extension("json.tmp");

        // Ретрай: файл может быть занят редактором или антивирусом.
        for _ in 0..3 {
            if std::fs::write(&tmp, format!("{json}\n")).is_ok()
                && std::fs::rename(&tmp, &self.write).is_ok()
            {
                return Ok(());
            }
            std::thread::sleep(Duration::from_millis(40));
        }
        let _ = std::fs::remove_file(&tmp);
        std::fs::write(&self.write, format!("{json}\n"))
    }
}

impl ServerList {
    /// Сохраняет список в `path`, возвращая текст ошибки для UI.
    pub fn save(&self, path: &ServersPath) -> Option<String> {
        path.save(&self.entries)
            .err()
            .map(|error| format!("Не удалось сохранить список серверов: {error}"))
    }
}

/// Конфиг-каталог приложения в стиле `dirs`/`directories`, без лишней зависимости.
fn config_dir() -> Option<PathBuf> {
    const APP: &str = "doppely";

    #[cfg(target_os = "windows")]
    let dir = Some(PathBuf::from(std::env::var_os("APPDATA")?).join(APP));

    #[cfg(target_os = "macos")]
    let dir = Some(
        PathBuf::from(std::env::var_os("HOME")?)
            .join("Library")
            .join("Application Support")
            .join(APP),
    );

    #[cfg(all(unix, not(target_os = "macos")))]
    let dir = {
        let base = std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| Some(PathBuf::from(std::env::var_os("HOME")?).join(".config")));
        base.map(|base| base.join(APP))
    };

    #[cfg(not(any(unix, target_os = "windows")))]
    let dir = None;

    dir
}

/// Никогда не показываем пустое имя в списке: адрес понятен и без него.
pub fn display_name(name: &str, address: &str) -> String {
    let name = name.trim();
    if name.is_empty() {
        address.to_string()
    } else {
        name.to_string()
    }
}

impl fmt::Display for Server {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.name, self.address)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_accepts_common_forms() {
        assert_eq!(parse_address("127.0.0.1").unwrap(), "127.0.0.1:5000");
        assert_eq!(parse_address(" 192.168.3.5 ").unwrap(), "192.168.3.5:5000");
        assert_eq!(
            parse_address("192.168.3.5:6000").unwrap(),
            "192.168.3.5:6000"
        );
        assert_eq!(parse_address("udp://[::1]:5000/").unwrap(), "[::1]:5000");
        assert_eq!(parse_address("localhost").unwrap(), LOCALHOST.to_string());
        assert!(parse_address("").is_err());
        assert!(parse_address("nope").is_err());
    }

    #[test]
    fn localhost_is_always_first_and_immutable() {
        let mut list = ServerList::default();
        assert_eq!(list.all().len(), 1);
        assert!(!list.remove(LOCAL_HOST_ID));

        list.insert("Home".into(), "192.168.3.5:5000".into());
        let all = list.all();
        assert!(all[0].local);
        assert_eq!(all[1].name, "Home");
        assert_eq!(all[1].id, 1);
        assert!(list.remove(1));
        assert_eq!(list.all().len(), 1);
    }

    #[test]
    fn duplicates_are_rejected() {
        let mut list = ServerList::default();
        assert!(!list.insert("Дубль".into(), LOCALHOST.to_string()));
        assert!(list.insert("Home".into(), "192.168.3.5:5000".into()));
        assert!(!list.insert("Home 2".into(), "192.168.3.5:5000".into()));
    }

    #[test]
    fn saves_only_non_local_entries() {
        let mut list = ServerList::default();
        list.insert("Home".into(), "192.168.3.5:5000".into());
        let json = serde_json::to_string(&list.entries).unwrap();
        assert!(!json.contains("127.0.0.1:5000"));
    }

    /// Пишет во временный каталог: так проверяются и атомарная запись, и
    /// чтение на старте, не трогая настоящий `assets/servers.json`.
    struct TempDir(PathBuf);

    impl TempDir {
        fn new(tag: &str) -> Self {
            let dir = std::env::temp_dir().join(format!(
                "doppely-{tag}-{}-{:?}",
                std::process::id(),
                std::thread::current().id()
            ));
            std::fs::create_dir_all(&dir).unwrap();
            Self(dir)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn temp_path(dir: &TempDir) -> ServersPath {
        let file = dir.path().join(SERVERS_FILE);
        ServersPath {
            read: file.clone(),
            write: file,
        }
    }

    #[test]
    fn round_trips_through_disk() {
        let dir = TempDir::new("roundtrip");
        let path = temp_path(&dir);

        // Нет файла — пустой список и созданный файл.
        let (entries, warning) = path.load();
        assert!(entries.is_empty() && warning.is_none());
        assert!(path.read_path().exists());

        let mut list = ServerList::with_servers(entries);
        assert!(list.insert("Home".into(), "192.168.3.5:5000".into()));
        assert_eq!(list.save(&path), None);

        let (entries, warning) = path.load();
        assert!(warning.is_none());
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "Home");

        // Localhost не попадает в файл, но есть в списке для UI.
        let list = ServerList::with_servers(entries);
        let all = list.all();
        assert_eq!(all.len(), 2);
        assert!(all[0].local);
    }

    #[test]
    fn corrupt_file_is_backed_up() {
        let dir = TempDir::new("corrupt");
        let path = temp_path(&dir);
        std::fs::write(path.read_path(), "{ not json").unwrap();

        let (entries, warning) = path.load();
        assert!(entries.is_empty());
        assert!(warning.is_some());
        assert!(path.read_path().with_extension("json.bak").exists());
        assert!(path.read_path().exists());
    }
}
