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