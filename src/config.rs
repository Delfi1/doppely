use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::{Path, PathBuf},
    str::FromStr,
};

use bevy::prelude::Resource;
use serde::{Deserialize, Deserializer, Serialize};

use crate::SharedSettings;

pub const CONFIG_FILE: &str = "config.json";

const DEFAULT_HOST: &str = "127.0.0.1";
const DEFAULT_BIND: &str = "0.0.0.0";
const DEFAULT_PORT: u16 = 5000;
const DEFAULT_PROTOCOL_ID: u64 = 1;

#[derive(Debug, Clone, Resource, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ServerConfig {
    pub host: String,
    #[serde(deserialize_with = "de_port")]
    pub port: u16,
    pub bind: String,
    pub password: String,
    pub protocol_id: u64,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: DEFAULT_HOST.to_string(),
            port: DEFAULT_PORT,
            bind: DEFAULT_BIND.to_string(),
            password: String::new(),
            protocol_id: DEFAULT_PROTOCOL_ID,
        }
    }
}

impl ServerConfig {
    pub fn local_addr(&self) -> SocketAddr {
        endpoint(&self.host, self.port, IpAddr::V4(Ipv4Addr::LOCALHOST))
    }

    pub fn bind_addr(&self) -> SocketAddr {
        endpoint(&self.bind, self.port, IpAddr::V4(Ipv4Addr::UNSPECIFIED))
    }

    pub fn private_key(&self) -> [u8; 32] {
        let mut key = [0u8; 32];
        let password = self.password.trim().as_bytes();
        if password.is_empty() {
            return key;
        }
        for (index, byte) in key.iter_mut().enumerate() {
            *byte = password[index % password.len()];
        }
        key
    }

    pub fn shared(&self) -> SharedSettings {
        SharedSettings {
            protocol_id: self.protocol_id,
            private_key: self.private_key(),
        }
    }

    pub fn path() -> PathBuf {
        asset_paths(CONFIG_FILE).0
    }

    pub fn read() -> (Self, PathBuf, Option<String>) {
        let path = Self::path();
        let (config, warning) = Self::load(&path);
        (config, path, warning)
    }

    pub fn load(path: &Path) -> (Self, Option<String>) {
        match std::fs::read_to_string(path) {
            Ok(data) => match serde_json::from_str(&data) {
                Ok(config) => (config, None),
                Err(error) => (
                    Self::default(),
                    Some(format!(
                        "Конфиг {} повреждён, взяты значения по умолчанию ({error})",
                        path.display()
                    )),
                ),
            },
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => (Self::default(), None),
            Err(error) => (
                Self::default(),
                Some(format!("Не удалось прочитать {}: {error}", path.display())),
            ),
        }
    }
}

fn de_port<'de, D>(deserializer: D) -> Result<u16, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Port {
        Number(u16),
        Text(String),
    }

    match Port::deserialize(deserializer)? {
        Port::Number(port) => Ok(port),
        Port::Text(text) => text.trim().parse().map_err(serde::de::Error::custom),
    }
}

fn endpoint(host: &str, port: u16, fallback: IpAddr) -> SocketAddr {
    let host = host.trim();
    if let Ok(addr) = SocketAddr::from_str(host) {
        return addr;
    }
    if let Ok(ip) = IpAddr::from_str(host) {
        return SocketAddr::new(ip, port);
    }
    if host.eq_ignore_ascii_case("localhost") {
        return SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port);
    }
    SocketAddr::new(fallback, port)
}

pub(crate) fn asset_paths(file: &str) -> (PathBuf, PathBuf) {
    let read = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join(file);

    if cfg!(debug_assertions) {
        return (read.clone(), read);
    }

    let write = config_dir()
        .map(|dir| dir.join(file))
        .unwrap_or_else(|| read.clone());
    (read, write)
}

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
