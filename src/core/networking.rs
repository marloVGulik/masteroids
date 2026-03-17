use std::net::{ToSocketAddrs, UdpSocket};

pub enum NetworkMessage {
    StartGame,
    Ready,
    Alive,
    ShareSeed { spawn_seed: u32, destroy_seed: u32 },
    AsteroidHit { size: u8 },
    Connect { name: String },
    TargetPlayer { id: u32 },
    UserAmount { amount: u8 },
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
    UserAmount = 7,
}

impl NetworkMessage {
    // Convert Enum to a Vec of bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            NetworkMessage::StartGame => vec![MessageId::StartGame as u8],
            NetworkMessage::Ready => vec![MessageId::Ready as u8],
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
            NetworkMessage::UserAmount { amount } => vec![MessageId::UserAmount as u8, *amount],
        }
    }

    // Convert bytes back into an Enum
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        let id = bytes.get(0)?;
        match *id {
            0 => Some(NetworkMessage::StartGame),
            1 => Some(NetworkMessage::Ready),
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
            7 if bytes.len() >= 2 => Some(NetworkMessage::UserAmount { amount: bytes[1] }),

            _ => None,
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
        let bytes = msg.to_bytes();
        let resolved_target_opt = NetworkManager::resolver(target);

        if let Some(resolved_target) = resolved_target_opt {
            let _ = self.socket.send_to(&bytes, resolved_target);
        }
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