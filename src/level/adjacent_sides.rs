use super::chunk_data::CHUNK_SIZE;

#[derive(Debug, Clone, Copy)]
pub struct AdjacentBlocks {
    pub left: bool,
    pub right: bool,
    pub top: bool,
    pub bottom: bool,
    pub front: bool,
    pub back: bool,
}

pub struct AdjacentChunkData {
    pub left: Option<[[bool; CHUNK_SIZE]; CHUNK_SIZE]>,
    pub right: Option<[[bool; CHUNK_SIZE]; CHUNK_SIZE]>,
    pub top: Option<[[bool; CHUNK_SIZE]; CHUNK_SIZE]>,
    pub bottom: Option<[[bool; CHUNK_SIZE]; CHUNK_SIZE]>,
    pub front: Option<[[bool; CHUNK_SIZE]; CHUNK_SIZE]>,
    pub back: Option<[[bool; CHUNK_SIZE]; CHUNK_SIZE]>,
}
