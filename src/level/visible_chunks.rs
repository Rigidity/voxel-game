use itertools::Itertools;

use crate::position::ChunkPos;

use super::chunk_data::CHUNK_SIZE;

pub fn visible_chunks(chunk_pos: ChunkPos, distance: i32) -> Vec<ChunkPos> {
    let center_pos = chunk_pos.center();
    let block_distance = distance * CHUNK_SIZE as i32;

    (-distance..=distance)
        .cartesian_product(-distance..=distance)
        .cartesian_product(-distance..=distance)
        .map(|((x, y), z)| chunk_pos + ChunkPos::new(x, y, z))
        .filter(|pos| center_pos.distance(pos.center()) <= block_distance as f32)
        .sorted_by(|a, b| {
            a.center()
                .distance(center_pos)
                .total_cmp(&b.center().distance(center_pos))
        })
        .collect_vec()
}
