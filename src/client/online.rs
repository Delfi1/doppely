use std::collections::HashMap;
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, UdpSocket};
use std::sync::{Arc, Mutex, mpsc};
use std::time::Duration;

use bevy::prelude::*;
use doppely::servers::ServerList;
use doppely::status::{STATUS_REQUEST, ServerStatusReply};

const POLL_INTERVAL: Duration = Duration::from_secs(2);
const PROBE_TIMEOUT: Duration = Duration::from_millis(350);

#[derive(Resource)]
pub struct ServerStatuses {
    targets: Arc<Mutex<Vec<SocketAddr>>>,
    results: Mutex<mpsc::Receiver<(String, Option<ServerStatusReply>)>>,
    by_address: HashMap<String, Option<ServerStatusReply>>,
}

impl ServerStatuses {
    pub fn start() -> Self {
        let targets = Arc::new(Mutex::new(Vec::new()));
        let (tx, rx) = mpsc::channel();
        let shared = Arc::clone(&targets);

        std::thread::spawn(move || {
            loop {
                let current = shared.lock().map(|t| t.clone()).unwrap_or_default();
                if current.is_empty() {
                    std::thread::sleep(Duration::from_millis(200));
                    continue;
                }
                for addr in current {
                    let result = probe(addr);
                    if tx.send((addr.to_string(), result)).is_err() {
                        return;
                    }
                }
                std::thread::sleep(POLL_INTERVAL);
            }
        });

        Self {
            targets,
            results: Mutex::new(rx),
            by_address: HashMap::new(),
        }
    }

    pub fn set_targets(&self, targets: Vec<SocketAddr>) {
        if let Ok(mut current) = self.targets.lock() {
            if *current != targets {
                *current = targets;
            }
        }
    }

    pub fn drain(&mut self) {
        let Ok(rx) = self.results.lock() else {
            return;
        };
        while let Ok((address, result)) = rx.try_recv() {
            self.by_address.insert(address, result);
        }
    }

    pub fn get(&self, address: &str) -> Option<&Option<ServerStatusReply>> {
        self.by_address.get(address)
    }
}

fn probe(addr: SocketAddr) -> Option<ServerStatusReply> {
    let bind = if addr.is_ipv4() {
        SocketAddr::from((Ipv4Addr::UNSPECIFIED, 0))
    } else {
        SocketAddr::from((Ipv6Addr::UNSPECIFIED, 0))
    };
    let socket = UdpSocket::bind(bind).ok()?;
    socket.set_read_timeout(Some(PROBE_TIMEOUT)).ok()?;
    socket
        .send_to(STATUS_REQUEST, ServerStatusReply::addr_for(addr))
        .ok()?;

    let mut buf = [0u8; 256];
    let (len, _) = socket.recv_from(&mut buf).ok()?;
    serde_json::from_slice(&buf[..len]).ok()
}

pub fn sync_targets(servers: Res<ServerList>, statuses: Res<ServerStatuses>) {
    if !servers.is_changed() {
        return;
    }
    statuses.set_targets(
        servers
            .all()
            .iter()
            .map(|server| server.socket_addr())
            .collect(),
    );
}

pub fn drain_results(mut statuses: ResMut<ServerStatuses>) {
    statuses.drain();
}
