#![allow(unused_imports)]
use anyhow::Context;
use serde::{Serialize, Deserialize};
use std::path::Path;
use sha1::{Digest, Sha1};
use tokio::fs::File;
use std::net::SocketAddrV4;
use std::path::PathBuf;
use crate::tracker::retrieve_peers;
use crate::peer::Handshake;
use tokio::io::{AsyncWriteExt, AsyncReadExt};
// use std::collections::HashSet;
use crate::download::DownloadState;

pub use hashes::Hashes;
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Torrent{
    pub announce: String,
    pub info: Info,
}

impl Torrent {
    pub fn info_hash(&self)->[u8;20]{
        let info_encoded = 
            serde_bencode::to_bytes(&self.info).expect("Re-encode?? IDK why but I gotta see later");// TODO
        let mut hasher = Sha1::new();
        hasher.update(&info_encoded);
        hasher.finalize().try_into().expect("GenericArray<_, 20> == [_; 20]")
    }

    pub async fn read(file:impl AsRef<Path>) -> anyhow::Result<Self>{
        let torr_file = tokio::fs::read(file).await.context("Reading torrent file")?;
        let t:Torrent = serde_bencode::from_bytes(&torr_file).context("Parsing torrent file")?;
        Ok(t)
    }

    pub fn length(&self)->usize{
        match &self.info.keys {
            Keys::SingleFile{length} => *length,
            Keys::MultiFile{files} => files.iter().map(|f| f.length).sum(),
        }
    }

    pub async fn download_all(&self,torr_path:&PathBuf, output: &PathBuf) -> anyhow::Result<()> {
        let peers = retrieve_peers(torr_path).await.unwrap();
        let mut state = DownloadState::new(self.info.pieces.0.len());
        
        let mut output_file = File::create(output.join(&self.info.name)).await?;
        
        for (piece_index, _) in self.info.pieces.0.iter().enumerate() {
            if !state.is_piece_complete(piece_index) {
                let piece_data = self.download_piece(&peers[0], piece_index).await?;
                output_file.write_all(&piece_data).await?;
                state.mark_piece_complete(piece_index);
                println!("Downloaded piece {}", piece_index);
            }
        }

        println!("Download complete!");
        Ok(())
    }

    async fn download_piece(&self, peer: &SocketAddrV4, piece_index: usize) -> anyhow::Result<Vec<u8>> {
        let mut stream = tokio::net::TcpStream::connect(peer).await?;
        
        // Perform handshake
        let mut handshake = Handshake::new(self.info_hash(), *b"Justorrent-alphatest");
        // ... (handshake code as in the Handshake command)
         {
                let handshake_bytes =
                    &mut handshake as *mut Handshake as *mut [u8; std::mem::size_of::<Handshake>()];
                // Safety: Handshake is a POD with repr(c) and repr(packed)
                let handshake_bytes: &mut [u8; std::mem::size_of::<Handshake>()] =
                    unsafe { &mut *handshake_bytes };
                peer.write_all(handshake_bytes)
                    .await
                    .context("write handshake")?;
                peer.read_exact(handshake_bytes)
                    .await
                    .context("read handshake")?;
            }
            assert_eq!(handshake.length, 19);
            assert_eq!(&handshake.bittorrent, b"BitTorrent protocol");
            

        // Request the piece
        let request = construct_request_message(piece_index, 0, self.info.plength);
        stream.write_all(&request).await?;

        // Read the piece
        let mut piece_data = Vec::new();
        stream.read_to_end(&mut piece_data).await?;

        Ok(piece_data)
    }

    pub fn construct_request_message(index: usize, begin: u32, length: u32) -> Vec<u8> {
    let mut message = Vec::new();
    message.extend_from_slice(&(13u32).to_be_bytes()); // Length prefix
    message.push(6u8); // Message ID for "request"
    message.extend_from_slice(&(index as u32).to_be_bytes());
    message.extend_from_slice(&begin.to_be_bytes());
    message.extend_from_slice(&length.to_be_bytes());
    message
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Info {
    pub name: String,

    #[serde(rename="piece length")]
    pub plength: usize,

    
    pub pieces: Hashes,
    #[serde(flatten)]
    pub keys: Keys,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Keys{
    SingleFile{
        length: usize, // Download is single filed, if length is present
    },

    MultiFile{
        files: Vec<TorrentFile>, // Otherwise, multifile as a directory structure
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TorrentFile{
    pub length: usize,

    pub path: Vec<String>,
}

mod hashes {
    use serde::de::{self, Deserialize, Deserializer, Visitor};
    use serde::ser::{Serialize, Serializer};
    use std::fmt;

    #[derive(Debug, Clone)]
    pub struct Hashes(pub Vec<[u8; 20]>);
    struct HashesVisitor;

    impl<'de> Visitor<'de> for HashesVisitor {
        type Value = Hashes;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a byte string whose length is a multiple of 20")
        }

        fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            if v.len() % 20 != 0 {
                return Err(E::custom(format!("length is {}", v.len())));
            }
            // TODO: use array_chunks when stable
            Ok(Hashes(
                v.chunks_exact(20)
                    .map(|slice_20| slice_20.try_into().expect("guaranteed to be length 20"))
                    .collect(),
            ))
        }
    }

    impl<'de> Deserialize<'de> for Hashes {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_bytes(HashesVisitor)
        }
    }

    impl Serialize for Hashes {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let single_slice = self.0.concat();
            serializer.serialize_bytes(&single_slice)
        }
    }
}
