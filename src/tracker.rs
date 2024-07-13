use anyhow::Context;
// use crate::torrent::Torrent;
use serde::{Deserialize, Serialize};
// use serde_bencode;
use peers::Peers;

use crate::torrent::Torrent;

#[derive(Debug,Clone,Serialize)]
pub struct TrackerRequest{
    pub peer_id: String,
    pub port: u16,
    pub uploaded: usize,
    pub downloaded: usize,
    pub left: usize,
    pub compact:u8,
}

#[derive(Debug,Clone,Deserialize)]
pub struct TrackerResponse{
    pub interval:usize, //Interval in seconds that the client should wait between sending regular requests to the tracker
    pub peers:Peers,
}

impl TrackerResponse{
    pub(crate) async fn query(t: &Torrent, info_hash: [u8; 20]) -> anyhow::Result<Self> {
        let request = TrackerRequest {
            peer_id: String::from("Justorrent-alphatest"),
            port: 6881,
            uploaded: 0,
            downloaded: 0,
            left: t.length(),
            compact: 1,
        };

        let url_params =
            serde_urlencoded::to_string(&request).context("url-encode tracker parameters")?;
        let tracker_url = format!(
            "{}?{}&info_hash={}",
            t.announce,
            url_params,
            &urlencode(&info_hash)
        );
        let response = reqwest::get(tracker_url).await.context("query tracker")?;
        let response = response.bytes().await.context("fetch tracker response")?;
        let tracker_info: TrackerResponse =
            serde_bencode::from_bytes(&response).context("parse tracker response")?;
        Ok(tracker_info)
    }
}


mod peers {
    use serde::de::{self, Deserialize, Deserializer, Visitor};
    use serde::ser::{Serialize, Serializer};
    use std::fmt;
    use std::net::{Ipv4Addr,SocketAddrV4};


    #[derive(Debug, Clone)]
    pub struct Peers(pub Vec<SocketAddrV4>);
    struct PeersVisitor;

    impl<'de> Visitor<'de> for PeersVisitor {
        type Value = Peers;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a byte string whose length is a multiple of 20")
        }

        fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            if v.len() % 6 != 0 {
                return Err(E::custom(format!("length is {}", v.len())));
            }
            // TODO: use array_chunks when stable
            Ok(Peers(
                v.chunks_exact(6)
                    .map(|slice_6| {
                        SocketAddrV4::new(
                            Ipv4Addr::new(slice_6[0],slice_6[1],slice_6[2],slice_6[3]),
                            u16::from_be_bytes([slice_6[4],slice_6[5]]),
                        ) 
                    }).collect(),
            
            ))
        }
    }

    impl<'de> Deserialize<'de> for Peers {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_bytes(PeersVisitor)
        }
    }

    impl Serialize for Peers {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let mut single_slice = Vec::with_capacity(6 * self.0.len());
            for peer in &self.0 {
                single_slice.extend(peer.ip().octets());
                single_slice.extend(peer.port().to_be_bytes());
            }
            serializer.serialize_bytes(&single_slice)
        }
    }
}

fn urlencode(t: &[u8; 20]) -> String {
    let mut encoded = String::with_capacity(3 * t.len());
    for &byte in t {
        encoded.push('%');
        encoded.push_str(&hex::encode(&[byte]));
    }
    encoded
}