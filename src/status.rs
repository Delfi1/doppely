use std::net::SocketAddr;

use serde::{Deserialize, Serialize};

use crate::ServerMode;

pub const STATUS_REQUEST: &[u8] = b"DPPSTAT1";

pub const STATUS_PORT_OFFSET: u16 = 1;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ServerStatusReply {
    pub players: u8,
    pub max_players: u8,
    pub mode: ServerMode,
}

impl ServerStatusReply {
    pub fn addr_for(server: SocketAddr) -> SocketAddr {
        SocketAddr::new(
            server.ip(),
            server.port().saturating_add(STATUS_PORT_OFFSET),
        )
    }
}
