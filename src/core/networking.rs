use std::net::{ToSocketAddrs, UdpSocket};

pub enum NetworkMessage {
    StartGame,
    Ready { is_ready: u8 },
    Alive,
    ShareSeed { spawn_seed: u32, destroy_seed: u32 },
    AsteroidHit { size: u8 },
    Connect { name: String },
    TargetPlayer { id: u32 },
    AttackPlayer { amount: u8 },
    UserAmount { amount: u8 },
    SummonAsteroid { x: f32, y: f32, direction: f32, speed: f32, size: u8 },
    UserData { id: u32, score: u32, health: u8, target_player: u32, name: String },
    Reject { reason: u8 },
    Accept { id: u32 },
    PlayerDamaged
}

#[repr(u8)]
enum MessageId {
    StartGame = 0,
    Ready = 1,
    Alive = 2,
    ShareSeed = 3,
    AsteroidHit = 4,
    Connect = 5,
    TargetPlayer = 6,
    AttackPlayer = 7,
    UserAmount = 8,
    SummonAsteroid = 9,
    UserData = 10,
    Reject = 11,
    Accept = 12,
    PlayerDamaged = 13
}

impl NetworkMessage {
    // Convert Enum to a Vec of bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            NetworkMessage::StartGame => vec![MessageId::StartGame as u8],
            NetworkMessage::Ready { is_ready } => {
                let mut bytes = vec![MessageId::Ready as u8];
                bytes.extend_from_slice(&is_ready.to_be_bytes());
                bytes
            },
            NetworkMessage::Alive => vec![MessageId::Alive as u8],
            NetworkMessage::ShareSeed { spawn_seed, destroy_seed } => {
                let mut bytes = vec![MessageId::ShareSeed as u8];
                bytes.extend_from_slice(&spawn_seed.to_be_bytes()); // Big Endian
                bytes.extend_from_slice(&destroy_seed.to_be_bytes());
                bytes
            },
            NetworkMessage::AsteroidHit { size } => vec![MessageId::AsteroidHit as u8, *size],
            NetworkMessage::Connect { name } => {
                let mut bytes = vec![MessageId::Connect as u8];
                bytes.extend_from_slice(name.as_bytes());
                bytes
            },
            NetworkMessage::TargetPlayer { id } => {
                let mut bytes = vec![MessageId::TargetPlayer as u8];
                bytes.extend_from_slice(&id.to_be_bytes());
                bytes
            },
            NetworkMessage::AttackPlayer { amount } => {
                let mut bytes = vec![MessageId::AttackPlayer as u8];
                bytes.extend_from_slice(&amount.to_be_bytes());
                bytes
            },
            NetworkMessage::UserAmount { amount } => vec![MessageId::UserAmount as u8, *amount],
            NetworkMessage::SummonAsteroid { x, y, direction, speed , size} => {
                let mut bytes = vec![MessageId::SummonAsteroid as u8];
                bytes.extend_from_slice(&x.to_be_bytes());
                bytes.extend_from_slice(&y.to_be_bytes());
                bytes.extend_from_slice(&direction.to_be_bytes());
                bytes.extend_from_slice(&speed.to_be_bytes());
                bytes.extend_from_slice(&size.to_be_bytes());
                bytes
            },
            NetworkMessage::UserData { id, score, health, target_player: target_id, name } => {
                let mut bytes = vec![MessageId::UserData as u8];
                bytes.extend_from_slice(&id.to_be_bytes());
                bytes.extend_from_slice(&score.to_be_bytes());
                bytes.extend_from_slice(&health.to_be_bytes());
                bytes.extend_from_slice(&target_id.to_be_bytes());
                bytes.extend_from_slice(name.as_bytes());
                bytes
            },
            NetworkMessage::Reject { reason } => {
                let mut bytes = vec![MessageId::Reject as u8];
                bytes.extend_from_slice(&reason.to_be_bytes());
                bytes
            },
            NetworkMessage::Accept { id } => {
                let mut bytes = vec![MessageId::Accept as u8];
                bytes.extend_from_slice(&id.to_be_bytes());
                bytes
            },
            NetworkMessage::PlayerDamaged => vec![MessageId::PlayerDamaged as u8]
        }
    }

    // Convert bytes back into an Enum
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        let id = bytes.get(0)?;

        // println!("Got netmsg ID {}", id);

        match *id {
            0 => Some(NetworkMessage::StartGame),
            1 if bytes.len() >= 2 => Some(NetworkMessage::Ready { is_ready: bytes[1] }),
            2 => Some(NetworkMessage::Alive),
            3 if bytes.len() >= 9 => {
                let spawn_seed = u32::from_be_bytes(bytes[1..5].try_into().ok()?);
                let destroy_seed = u32::from_be_bytes(bytes[5..9].try_into().ok()?);
                Some(NetworkMessage::ShareSeed { spawn_seed, destroy_seed })
            },
            4 if bytes.len() >= 2 => Some(NetworkMessage::AsteroidHit { size: bytes[1] }),
            5 if bytes.len() >= 2 => {
                let name = String::from_utf8(bytes[1..].to_vec()).ok()?;
                Some(NetworkMessage::Connect { name })
            },
            6 if bytes.len() >= 5 => {
                let id = u32::from_be_bytes(bytes[1..5].try_into().ok()?);
                Some(NetworkMessage::TargetPlayer { id })
            },
            7 if bytes.len() >= 2 => Some(NetworkMessage::AttackPlayer { amount: bytes[1] }),
            8 if bytes.len() >= 2 => Some(NetworkMessage::UserAmount { amount: bytes[1] }),
            9 if bytes.len() >= 18 => {
                let x = f32::from_be_bytes(bytes[1..5].try_into().ok()?);
                let y = f32::from_be_bytes(bytes[5..9].try_into().ok()?);
                let direction = f32::from_be_bytes(bytes[9..13].try_into().ok()?);
                let speed = f32::from_be_bytes(bytes[13..17].try_into().ok()?);
                let size = u8::from_be_bytes(bytes[17..18].try_into().ok()?);
                
                Some(NetworkMessage::SummonAsteroid { 
                    x, 
                    y, 
                    direction, 
                    speed, 
                    size 
                })
            },
            10 if bytes.len() >= 12 => {
                let id = u32::from_be_bytes(bytes[1..5].try_into().ok()?);
                let score = u32::from_be_bytes(bytes[5..9].try_into().ok()?);
                let health = u8::from_be_bytes(bytes[9..10].try_into().ok()?);
                let target_id = u32::from_be_bytes(bytes[10..14].try_into().ok()?);
                let name = String::from_utf8(bytes[14..].to_vec()).ok()?;
                Some(NetworkMessage::UserData { 
                    id, 
                    score,
                    health,
                    target_player: target_id,
                    name,
                })
            },
            11 if bytes.len() >= 2 => Some(NetworkMessage::Reject { reason: bytes[1] }),
            12 if bytes.len() >= 5 => Some(NetworkMessage::Accept { id: u32::from_be_bytes(bytes[1..4].try_into().ok()?) }),
            13 if bytes.len() >= 1 => Some(NetworkMessage::PlayerDamaged),
            _ => {
                println!("Failed message with id {}", id);
                None
            }
        }
    }
}

pub struct NetworkManager {
    socket: UdpSocket,
}

impl NetworkManager {
    pub fn new(addr: &str) -> Self {
        let res_addr = NetworkManager::resolver(addr);
        let final_addr: std::net::SocketAddr;
        if let Some(ok_res_addr) = res_addr {
            final_addr = ok_res_addr;
        } else {
            final_addr = std::net::SocketAddr::new(std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)), 42069);
        }

        let socket = UdpSocket::bind(
            format!("{}:{}", final_addr.ip().to_string(), final_addr.port().to_string())).expect("Couldn't bind to address");
        socket.set_nonblocking(true).ok();
        
        Self {
            socket
        }
    }

    pub fn emit(&self, target: &str, msg: &NetworkMessage) {
        let resolved_target_opt = NetworkManager::resolver(target);

        if let Some(resolved_target) = resolved_target_opt {
            self.emit_socket(&resolved_target, msg);
        }
    }
    pub fn emit_socket(&self, target: &std::net::SocketAddr, msg: &NetworkMessage) {
        let _ = self.socket.send_to(&msg.to_bytes(), target);
    }

    pub fn process_incoming(&self, mut handler: impl FnMut(std::net::SocketAddr, &NetworkMessage)) {
        let mut buf = [0u8; 1024];
        while let Ok((size, src)) = self.socket.recv_from(&mut buf) {
            if let Some(msg) = NetworkMessage::from_bytes(&buf[..size]) {
                handler(src, &msg);
            }
        }
    }

    fn resolver(addr: &str) -> Option<std::net::SocketAddr> {
        let resolved_addr = addr.to_socket_addrs();
        if let Ok(all_ips) = resolved_addr {
            let collected_ips: Vec<_> = all_ips.collect();

            // Prefer IPv4 for local testing
            if let Some(ipv4_addr) = collected_ips.iter().find(|ip| ip.is_ipv4()) {
                return Some(*ipv4_addr);
            }

            return collected_ips.get(0).copied();
        }

        return None;
    }
}