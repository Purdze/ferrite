use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct ServerEntry {
    pub name: String,
    pub address: String,
}

#[derive(Clone)]
pub enum PingState {
    Pinging,
    Success {
        motd: String,
        online: i32,
        max: i32,
        latency_ms: u64,
        #[allow(dead_code)]
        version: String,
    },
    Failed(String),
}

pub struct ServerList {
    pub servers: Vec<ServerEntry>,
    path: PathBuf,
}

pub type PingResults = Arc<RwLock<HashMap<String, PingState>>>;

impl ServerList {
    pub fn load(game_dir: &Path) -> Self {
        let path = game_dir.join("pomc_servers.json");
        let servers = std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
        Self { servers, path }
    }

    pub fn save(&self) {
        if let Ok(json) = serde_json::to_string_pretty(&self.servers) {
            if let Err(e) = std::fs::write(&self.path, json) {
                log::warn!("Failed to save server list: {e}");
            }
        }
    }

    pub fn add(&mut self, entry: ServerEntry) {
        self.servers.push(entry);
        self.save();
    }

    pub fn update(&mut self, index: usize, entry: ServerEntry) {
        if index < self.servers.len() {
            self.servers[index] = entry;
            self.save();
        }
    }

    pub fn remove(&mut self, index: usize) {
        if index < self.servers.len() {
            self.servers.remove(index);
            self.save();
        }
    }
}

pub fn ping_all_servers(
    rt: &tokio::runtime::Runtime,
    servers: &[ServerEntry],
    results: &PingResults,
) {
    for server in servers {
        let address = server.address.clone();
        results.write().insert(address.clone(), PingState::Pinging);
        let results = Arc::clone(results);
        rt.spawn(ping_server(address, results));
    }
}

async fn ping_server(address: String, results: PingResults) {
    use azalea_protocol::connect::Connection;
    use azalea_protocol::packets::handshake::s_intention::ServerboundIntention;
    use azalea_protocol::packets::status::s_ping_request::ServerboundPingRequest;
    use azalea_protocol::packets::status::s_status_request::ServerboundStatusRequest;
    use azalea_protocol::packets::status::ClientboundStatusPacket;
    use azalea_protocol::packets::{ClientIntention, PROTOCOL_VERSION};

    let result = async {
        let addr = resolve_address(&address)?;
        let mut conn: Connection<_, _> = Connection::new(&addr).await
            .map_err(|e| format!("Connection failed: {e}"))?;

        conn.write(ServerboundIntention {
            protocol_version: PROTOCOL_VERSION,
            hostname: addr.ip().to_string(),
            port: addr.port(),
            intention: ClientIntention::Status,
        })
        .await
        .map_err(|e| format!("Handshake failed: {e}"))?;

        let mut conn = conn.status();

        conn.write(ServerboundStatusRequest {})
            .await
            .map_err(|e| format!("Status request failed: {e}"))?;

        let packet = conn.read().await.map_err(|e| format!("Read failed: {e}"))?;
        let status = match packet {
            ClientboundStatusPacket::StatusResponse(s) => s,
            _ => return Err("Unexpected packet".to_string()),
        };

        let ping_start = Instant::now();
        let time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        conn.write(ServerboundPingRequest { time })
            .await
            .map_err(|e| format!("Ping request failed: {e}"))?;

        let _ = conn.read().await.map_err(|e| format!("Pong failed: {e}"))?;
        let latency_ms = ping_start.elapsed().as_millis() as u64;

        let motd = format!("{}", status.description);
        let version = status.version.name.clone();
        let (online, max) = (status.players.online, status.players.max);

        Ok(PingState::Success {
            motd,
            online,
            max,
            latency_ms,
            version,
        })
    }
    .await;

    let state = match result {
        Ok(s) => s,
        Err(e) => PingState::Failed(e),
    };
    results.write().insert(address, state);
}

fn with_default_port(address: &str) -> String {
    if address.contains(':') {
        address.to_string()
    } else {
        format!("{address}:25565")
    }
}

fn resolve_address(server: &str) -> Result<SocketAddr, String> {
    use std::net::ToSocketAddrs;

    let addr = with_default_port(server);
    addr.to_socket_addrs()
        .map_err(|e| format!("{addr}: {e}"))?
        .next()
        .ok_or_else(|| format!("{addr}: no addresses found"))
}

pub fn is_valid_address(address: &str) -> bool {
    if address.is_empty() {
        return false;
    }
    let with_port = with_default_port(address);
    with_port.parse::<SocketAddr>().is_ok()
        || with_port.split(':').next().is_some_and(|host| !host.is_empty())
}
