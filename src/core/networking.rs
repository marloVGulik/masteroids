use std::{net::UdpSocket, os::unix::net::SocketAddr};

pub enum NetworkMessage {
    StartGame,
    Ready,
    Alive,
    ShareSeed { spawn_seed: u32, destroy_seed: u32 },
    AsteroidHit { size: u8 },
    Connect { name: String },
}

#[repr(u8)]
enum MessageId {
    StartGame = 0,
    Ready = 1,
    Alive = 2,
    ShareSeed = 3,
    AsteroidHit = 4,
    Connect = 5,
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

            _ => None,
        }
    }
}

pub struct NetworkManager {
    socket: UdpSocket,
}

impl NetworkManager {
    pub fn new(addr: &str) -> Self {
        let socket = UdpSocket::bind(addr).expect("Bind failed");
        socket.set_nonblocking(true).ok();
        Self { socket }
    }

    pub fn emit(&self, target: &str, msg: NetworkMessage) {
        let bytes = msg.to_bytes();
        let _ = self.socket.send_to(&bytes, target);
    }

    pub fn process_incoming(&self, mut handler: impl FnMut(std::net::SocketAddr, NetworkMessage)) {
        let mut buf = [0u8; 1024];
        while let Ok((size, src)) = self.socket.recv_from(&mut buf) {
            if let Some(msg) = NetworkMessage::from_bytes(&buf[..size]) {
                handler(src, msg);
            }
        }
    }
}