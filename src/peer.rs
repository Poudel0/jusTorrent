use std::{net::{SocketAddrV4}, sync::Arc};
// use tokio::net::TcpStream;
// use anyhow::Context;
// use tokio::io::{AsyncReadExt, AsyncWriteExt};

// use futures_util::lock::Mutex;

// use crate::{download::DownloadState, torrent::Torrent};

// pub struct Handshake{
//     pstr:String,
//     info_hash:[u8;20],
//     peer_id:[u8;20]
// }

// impl Handshake{
//     pub fn new(info_hash:[u8;20],peer_id:[u8;20]) -> Self{
//         Self {  pstr: "BitTorrent protocol".to_string(),
//             info_hash,
//             peer_id,
//          }
//     }

//     pub fn as_bytes(&self) -> Vec<u8>{
//         let mut bytes = Vec::new();
//         bytes.push(self.pstr.len() as u8);
//         bytes.extend_from_slice(self.pstr.as_bytes());
//         bytes.extend_from_slice(&[0u8; 8]); // reserved
//         bytes.extend_from_slice(&self.info_hash);
//         bytes.extend_from_slice(&self.peer_id);
//         bytes
//     }

//     pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
//         if bytes.len() != 68 {
//             return None;
//         }
//         let pstr_len = bytes[0] as usize;
//         let pstr = String::from_utf8(bytes[1..1 + pstr_len].to_vec()).ok()?;
//         let info_hash = bytes[28..48].try_into().ok()?;
//         let peer_id = bytes[48..68].try_into().ok()?;
//         Some(Self {
//             pstr,
//             info_hash,
//             peer_id,
//         })
//     }

// }

#[repr(packed)]
pub struct Handshake {
    pub length: u8,
    pub bittorrent: [u8; 19],
    pub reserved: [u8; 8],
    pub info_hash: [u8; 20],
    pub peer_id: [u8; 20],
}

impl Handshake {
    pub fn new(info_hash: [u8; 20], peer_id: [u8; 20]) -> Self {
        Self {
            length: 19,
            bittorrent: *b"BitTorrent protocol",
            reserved: [0; 8],
            info_hash,
            peer_id,
        }
    }

    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        let bytes = self as *mut Self as *mut [u8; std::mem::size_of::<Self>()];
        // Safety: Self is a POD with repr(c) and repr(packed)
        let bytes: &mut [u8; std::mem::size_of::<Self>()] = unsafe { &mut *bytes };
        bytes
    }
}

// pub async fn connect_to_peer(peer:SocketAddrV4,torrent:&Torrent)-> anyhow::Result<()>{
//     let mut stream = TcpStream::connect(peer).await.context("Connect to peer")?;

//     // Handshake
//     let info_hash = torrent.info_hash();
//     let peer_id = [0u8; 20]; // Generate a peer_id for your client
//     let handshake = Handshake::new(info_hash, peer_id);
//     stream.write_all(&handshake.as_bytes()).await.context("Send handshake")?;
//     println!("Sent handshake: {:?}", handshake.as_bytes());

//     // Read and verify handshake response
//     let mut buffer = [0u8; 68];
//     stream.read_exact(&mut buffer).await.context("Read handshake response")?;
//     let response = Handshake::from_bytes(&buffer).context("Parse handshake response")?;
//     println!("Received handshake: {:?}", response.as_bytes());

//     if response.info_hash != info_hash {
//         return Err(anyhow::anyhow!("Received info_hash does not match"));
//     }

//     Ok(())
// }