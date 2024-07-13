use serde_json;
use clap::{Parser, Subcommand};
use sha1::digest::typenum::Length;
use std::path::PathBuf;
use anyhow::Context;
use serde_bencode;
use justorrent::{torrent::{self,Torrent}, tracker::{TrackerRequest, TrackerResponse}};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use futures_util::{SinkExt, StreamExt};



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
            let torrr_file = std::fs::read(torrent).context("The torrent file is read")?;
            let t:Torrent  = serde_bencode::from_bytes(&torrr_file).context("PArse the torrrent file?")?;
            
            let length = if let torrent::Keys::SingleFile { length } = t.info.keys {
                length
            } else {
                todo!();
            };

            let info_hash = t.info_hash();
            let request = TrackerRequest{
                peer_id: String::from("Justorrent-alphatest"),
                port: 6881,
                uploaded:0,
                downloaded:0,
                left:length,
                compact:1,
            };
            let url_params = serde_urlencoded::to_string(&request).context("url encoded tracker params")?;
            let tracker_url = format!("{}?{}&info_hash={}",
                t.announce,
                url_params,
                urlencode(&info_hash)
            );
            let response = reqwest::get(tracker_url).await.context("Reesponse frm tracker")?;
            let response = response.bytes().await.context("Reesponse")?;
            let response: TrackerResponse = serde_bencode::from_bytes(&response).context("Parse tracker response")?;
            for peer in  response.peers.0{
                println!("{}:{}", peer.ip(),peer.port());
            }


        }
        Command::Handshake { torrent, peer }=>{}

        Command::Download { output, torrent }=>{}
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
