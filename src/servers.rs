use std::{
    fmt,
    net::{IpAddr, SocketAddr},
    path::{Path, PathBuf},
    str::FromStr,
    time::Duration,
};

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::config::{ServerConfig, asset_paths};

pub const SERVERS_FILE: &str = "servers.json";

pub const LOCAL_HOST_ID: u32 = 0;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct ServerEntry {
    pub name: String,
    pub address: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Server {
    pub id: u32,
    pub name: String,
    pub address: String,
    pub local: bool,
}

impl Server {
    pub fn local(addr: SocketAddr) -> Self {
        Self {
            id: LOCAL_HOST_ID,
            name: "Localhost".to_string(),
            address: addr.to_string(),
            local: true,
        }
    }

    pub fn socket_addr(&self) -> SocketAddr {
        self.address
            .parse()
            .expect("адреса в списке серверов валидируются при загрузке и добавлении")
    }

    pub fn entry(&self) -> Option<ServerEntry> {
        (!self.local).then(|| ServerEntry {
            name: self.name.clone(),
            address: self.address.clone(),
        })
    }
}

#[derive(Debug, Clone, Resource)]
pub struct ServerList {
    entries: Vec<ServerEntry>,
    local: SocketAddr,
}

impl Default for ServerList {
    fn default() -> Self {
        Self::with_servers(Vec::new(), ServerConfig::default().local_addr())
    }
}

impl ServerList {
    pub fn with_servers(entries: Vec<ServerEntry>, local: SocketAddr) -> Self {
        Self { entries, local }
    }

    pub fn all(&self) -> Vec<Server> {
        let mut servers = Vec::with_capacity(self.entries.len() + 1);
        servers.push(Server::local(self.local));
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

    pub fn insert(&mut self, name: String, address: String) -> bool {
        if address == self.local.to_string()
            || self.entries.iter().any(|entry| entry.address == address)
        {
            return false;
        }
        self.entries.push(ServerEntry { name, address });
        true
    }

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

pub fn parse_address(input: &str, config: &ServerConfig) -> Result<String, String> {
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
        return Ok(SocketAddr::new(ip, config.port).to_string());
    }
    if raw.eq_ignore_ascii_case("localhost") {
        return Ok(config.local_addr().to_string());
    }
    Err(format!("Не удалось разобрать адрес «{raw}»"))
}

#[derive(Debug, Clone, Resource)]
pub struct ServersPath {
    read: PathBuf,
    write: PathBuf,
}

impl ServersPath {
    pub fn resolve() -> Self {
        let (read, write) = asset_paths(SERVERS_FILE);
        Self { read, write }
    }

    pub fn read_path(&self) -> &Path {
        &self.read
    }

    pub fn write_path(&self) -> &Path {
        &self.write
    }

    pub fn active_path(&self) -> &Path {
        if self.write.exists() {
            &self.write
        } else {
            &self.read
        }
    }

    pub fn load(&self, config: &ServerConfig) -> (Vec<ServerEntry>, Option<String>) {
        let path = self.active_path();
        let data = match std::fs::read_to_string(path) {
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
                    Some(format!("Не удалось прочитать {}: {error}", path.display())),
                );
            }
        };

        let result: Result<Vec<ServerEntry>, _> = serde_json::from_str(&data);
        match result {
            Ok(entries) => {
                let mut seen = vec![config.local_addr().to_string()];
                let entries: Vec<_> = entries
                    .into_iter()
                    .filter(|entry| {
                        let addr = parse_address(&entry.address, config).unwrap_or_default();
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
                let backup = path.with_extension("json.bak");
                let _ = std::fs::rename(path, &backup);
                let _ = self.save(&[]);
                bevy::log::warn!("{error}, файл отложен в {:?}", backup);
                (
                    Vec::new(),
                    Some(format!("Список серверов повреждён, начат заново ({error})")),
                )
            }
        }
    }

    pub fn save(&self, entries: &[ServerEntry]) -> std::io::Result<()> {
        if let Some(parent) = self.write.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(entries)
            .map_err(|error| std::io::Error::other(error.to_string()))?;
        let tmp = self.write.with_extension("json.tmp");

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
    pub fn save(&self, path: &ServersPath) -> Option<String> {
        path.save(&self.entries)
            .err()
            .map(|error| format!("Не удалось сохранить список серверов: {error}"))
    }
}

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
