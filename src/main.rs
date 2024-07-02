use serde_json;
use clap::{Parser, Subcommand};
use sha1::digest::typenum::Length;
use std::path::PathBuf;
use anyhow::Context;
use serde_bencode;
use justorrent::torrent::{self,Torrent};
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
        Command::Peers { torrent }=>{}
        Command::Handshake { torrent, peer }=>{}
        _=>{}

       
    }
    Ok(())
}
