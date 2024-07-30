#![allow(unused_imports)]
use std::{collections::{HashSet,HashMap}, net::SocketAddrV4};
use crate::torrent::{Torrent,Keys};
use serde;


#[derive(Default)]
pub struct DownloadState {
    downloaded_pieces: HashSet<usize>,
    total_pieces: usize,
}

impl DownloadState {
    pub fn new(total_pieces: usize) -> Self {
        Self {
            downloaded_pieces: HashSet::new(),
            total_pieces,
        }
    }

    pub fn is_complete(&self) -> bool {
        self.downloaded_pieces.len() == self.total_pieces
    }

    pub fn mark_piece_complete(&mut self, piece_index: usize) {
        self.downloaded_pieces.insert(piece_index);
    }

    pub fn is_piece_complete(&self, piece_index: usize) -> bool {
        self.downloaded_pieces.contains(&piece_index)
    }
}
// #[derive(Debug)]
// pub struct DownloadState{
//     peers: Vec<SocketAddrV4>,
//     connected_peers: HashSet<SocketAddrV4>,
//     pieces: Vec<bool>,
//     piece_availability: HashMap<usize, HashSet<SocketAddrV4>>,
//     downloaded_pieces:usize,
//     total_pieces:usize,
//     piece_length:usize,
//     total_length:usize,
// }

// impl DownloadState{
//     pub fn new(torrent:&Torrent, peers: Vec<SocketAddrV4>)->Self{
//         let total_pieces = torrent.info.pieces.0.len();
//         let piece_length = torrent.info.plength;
//         let total_length = match &torrent.info.keys{
//             Keys::SingleFile{length} => *length,
//             Keys::MultiFile{files} => files.iter().map(|f| f.length).sum(),
//         };
//         Self { 
//             peers,
//             connected_peers: HashSet::new(),
//             pieces:vec![false;total_pieces],
//             piece_availability: HashMap::new(),
//             downloaded_pieces: 0,
//             total_pieces,
//             piece_length,
//             total_length
//         }

//     }

//     pub fn mark_piece_available(&mut self,piece_index:usize, peer: SocketAddrV4){
//         self.piece_availability
//         .entry(piece_index)
//         .or_insert_with(HashSet::new)
//         .insert(peer);
//     }

//     pub fn mark_piece_downloaded(&mut self,piece_index:usize){
//         if !self.pieces[piece_index]{
//             self.pieces[piece_index] = true;
//             self.downloaded_pieces += 1;
//         }

//     }
//     pub fn is_completed(&self)->bool{
//         self.downloaded_pieces == self.total_pieces
//     }
// }



