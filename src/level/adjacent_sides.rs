#[derive(Debug, Clone, Copy)]
pub struct AdjacentBlocks {
    pub left: bool,
    pub right: bool,
    pub top: bool,
    pub bottom: bool,
    pub front: bool,
    pub back: bool,
}
