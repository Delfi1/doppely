use std::net::{SocketAddr, UdpSocket};
use std::sync::{
    Arc,
    atomic::{AtomicU8, Ordering},
};

use bevy::prelude::*;
use doppely::status::{STATUS_REQUEST, ServerStatusReply};
use doppely::{MAX_PLAYERS, ServerMode};

#[derive(Default)]
struct Live {
    players: AtomicU8,
    mode: AtomicU8,
}

#[derive(Resource, Clone, Default)]
pub struct ServerStatus(Arc<Live>);

impl ServerStatus {
    pub fn set(&self, players: u8, mode: ServerMode) {
        self.0.players.store(players, Ordering::Relaxed);
        self.0.mode.store(mode_code(mode), Ordering::Relaxed);
    }

    fn reply(&self) -> ServerStatusReply {
        ServerStatusReply {
            players: self.0.players.load(Ordering::Relaxed),
            max_players: MAX_PLAYERS,
            mode: mode_from(self.0.mode.load(Ordering::Relaxed)),
        }
    }
}

fn mode_code(mode: ServerMode) -> u8 {
    match mode {
        ServerMode::Lobby => 0,
        ServerMode::Game => 1,
        ServerMode::Edit => 2,
    }
}

fn mode_from(code: u8) -> ServerMode {
    match code {
        1 => ServerMode::Game,
        2 => ServerMode::Edit,
        _ => ServerMode::Lobby,
    }
}

pub fn spawn(addr: SocketAddr, status: ServerStatus) {
    let socket = match UdpSocket::bind(addr) {
        Ok(socket) => socket,
        Err(error) => {
            bevy::log::warn!("Статус-порт {addr} недоступен: {error}");
            return;
        }
    };
    bevy::log::info!("Статус сервера: {addr}");

    std::thread::spawn(move || {
        let mut buf = [0u8; 64];
        loop {
            let Ok((len, peer)) = socket.recv_from(&mut buf) else {
                continue;
            };
            if &buf[..len] != STATUS_REQUEST {
                continue;
            }
            if let Ok(reply) = serde_json::to_vec(&status.reply()) {
                let _ = socket.send_to(&reply, peer);
            }
        }
    });
}
