#![allow(unused_imports)]
use serde_json;
use clap::{Parser, Subcommand};
use sha1::digest::typenum::Length;
use std::{net::SocketAddrV4, path::PathBuf, sync::Arc};
use anyhow::Context;
use serde_bencode;
use justorrent::{peer::{Handshake}, torrent::{self,Torrent}, tracker::{self, TrackerRequest, TrackerResponse, retrieve_peers,},download::DownloadState};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use futures_util::{SinkExt, StreamExt};
use tokio::fs::File;
use std::collections::HashSet;


// use



#[derive(Parser,Debug)]
#[command(author, version, about, long_about = None)]
struct Args{
    #[command(subcommand)]
    command:Command,
}

#[derive(Subcommand,Debug)]
#[clap(rename_all = "snake_case")]
enum Command {
    Info{
        torrent: PathBuf,
    },
    Peers{
        torrent: PathBuf,
    },
    Handshake{
        torrent: PathBuf,
        peer:String,
    },
    Download{
        output:PathBuf,
        torrent: PathBuf,
    }

}



// fn decode_bencoded_value(encoded_value: &str)-> (serde_json::Value,&str){
//     match encoded_value.chars().next(){

//     }
// }
#[tokio::main]
async fn main()-> anyhow::Result<()> {
    let args = Args::parse();

    match args.command{
        Command::Info{torrent}=> {
            let torr_file = std::fs::read(torrent).context("The torr file is read")?;
            let t: Torrent = serde_bencode::from_bytes(&torr_file).context("Parseeee torrent File")?;
            
            println!("Tracker URL: {}",t.announce);
            let length = if let torrent::Keys::SingleFile { length } = t.info.keys{
                length
            } else {
                todo!();
            };
            println!("Length: {length}");
            let info_hash = t.info_hash();
            println!("Info hash: {}",hex::encode(&info_hash));
            for hash in t.info.pieces.0{
                println!("{}", hex::encode(&hash));
            }
        }
        Command::Peers { torrent }=>{
           if let Some(peers) = retrieve_peers(&torrent).await{
            for peer in peers{
                println!("{}", peer);
            }
           } else{
            eprint!("Failed to retrieve peers");
           }
        }
    Command::Handshake { torrent, peer } => {
        // let t = Torrent::read(&torrent).await.context("Reading torrent file")?;
        // // let state = Arc::new(Mutex::new(DownloadState::new()));
        // // let state
        // let peers = retrieve_peers(torrent.clone()).await.context("Retrieving peers")?;
        // if let Some(peer) = peers.first() {
        //     let addr = peer.parse().context("Parsing peer address")?;
        //     connect_to_peer(addr, &t).await.context("Connecting to peer")?;
        // } else {
        //     println!("No peers found");
        // }
            let t = Torrent::read(&torrent).await?;

            let info_hash = t.info_hash();
            let peer = peer.parse::<SocketAddrV4>().context("parse peer address")?;
            let mut peer = tokio::net::TcpStream::connect(peer)
                .await
                .context("connect to peer")?;
            let mut handshake = Handshake::new(info_hash, *b"Justorrent-alphatest");
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
            println!("Peer ID: {}", hex::encode(&handshake.peer_id));
   
    }


    Command::Download {  torrent, output }=>{   
        // let torr_file = Torrent::read(torrent).await?;

        // // if !output.exists(){
        // //     tokio::fs::create_dir_all(&output).await?;
        // // }

        // let files = torr_file.download_all(&output).await;

        let torr_file = Torrent::read(&torrent).await?;
        let download_state = torr_file.download_all(&torrent, &output).await?;
        println!("Download completed. Final state: {:?}", download_state);


    }
    
    _=>{}

       
    }
    Ok(())
}


fn urlencode(t: &[u8; 20]) -> String {
    let mut encoded = String::with_capacity(3 * t.len());
    for &byte in t {
        encoded.push('%');
        encoded.push_str(&hex::encode(&[byte]));
    }
    encoded
}
