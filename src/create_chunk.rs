use bevy::{
    prelude::Mesh,
    render::{mesh, render_resource::PrimitiveTopology},
};

use crate::Chunk;

pub fn create_mesh(chunk: &Chunk) -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    let mut positions = Vec::new();
    let mut texcoords = Vec::new();
    let mut normals = Vec::new();
    let mut indices = Vec::new();

    for x in 0..16 {
        for y in 0..16 {
            for z in 0..16 {
                if !chunk.blocks[x][y][z] {
                    continue;
                }

                let (x, y, z) = (x as f32, y as f32, z as f32);
                let mut index = positions.len() as u32;

                // Left
                positions.push([x - 0.5, y - 0.5, z - 0.5]);
                positions.push([x - 0.5, y + 0.5, z - 0.5]);
                positions.push([x - 0.5, y + 0.5, z + 0.5]);
                positions.push([x - 0.5, y - 0.5, z + 0.5]);

                normals.push([1.0, 0.0, 0.0]);
                normals.push([1.0, 0.0, 0.0]);
                normals.push([1.0, 0.0, 0.0]);
                normals.push([1.0, 0.0, 0.0]);

                texcoords.push([0.0, 0.0]);
                texcoords.push([0.0, 1.0]);
                texcoords.push([1.0, 1.0]);
                texcoords.push([1.0, 0.0]);

                indices.push(index);
                indices.push(index + 3);
                indices.push(index + 2);
                indices.push(index + 2);
                indices.push(index + 1);
                indices.push(index);

                index += 4;

                // Right
                positions.push([x + 0.5, y - 0.5, z - 0.5]);
                positions.push([x + 0.5, y + 0.5, z - 0.5]);
                positions.push([x + 0.5, y + 0.5, z + 0.5]);
                positions.push([x + 0.5, y - 0.5, z + 0.5]);

                normals.push([-1.0, 0.0, 0.0]);
                normals.push([-1.0, 0.0, 0.0]);
                normals.push([-1.0, 0.0, 0.0]);
                normals.push([-1.0, 0.0, 0.0]);

                texcoords.push([0.0, 0.0]);
                texcoords.push([0.0, 1.0]);
                texcoords.push([1.0, 1.0]);
                texcoords.push([1.0, 0.0]);

                indices.push(index);
                indices.push(index + 1);
                indices.push(index + 2);
                indices.push(index + 2);
                indices.push(index + 3);
                indices.push(index);

                index += 4;

                // Top
                positions.push([x - 0.5, y + 0.5, z - 0.5]);
                positions.push([x + 0.5, y + 0.5, z - 0.5]);
                positions.push([x + 0.5, y + 0.5, z + 0.5]);
                positions.push([x - 0.5, y + 0.5, z + 0.5]);

                normals.push([0.0, 1.0, 0.0]);
                normals.push([0.0, 1.0, 0.0]);
                normals.push([0.0, 1.0, 0.0]);
                normals.push([0.0, 1.0, 0.0]);

                texcoords.push([0.0, 0.0]);
                texcoords.push([0.0, 1.0]);
                texcoords.push([1.0, 1.0]);
                texcoords.push([1.0, 0.0]);

                indices.push(index);
                indices.push(index + 3);
                indices.push(index + 2);
                indices.push(index + 2);
                indices.push(index + 1);
                indices.push(index);

                index += 4;

                // Bottom
                positions.push([x - 0.5, y - 0.5, z - 0.5]);
                positions.push([x + 0.5, y - 0.5, z - 0.5]);
                positions.push([x + 0.5, y - 0.5, z + 0.5]);
                positions.push([x - 0.5, y - 0.5, z + 0.5]);

                normals.push([0.0, -1.0, 0.0]);
                normals.push([0.0, -1.0, 0.0]);
                normals.push([0.0, -1.0, 0.0]);
                normals.push([0.0, -1.0, 0.0]);

                texcoords.push([0.0, 0.0]);
                texcoords.push([0.0, 1.0]);
                texcoords.push([1.0, 1.0]);
                texcoords.push([1.0, 0.0]);

                indices.push(index);
                indices.push(index + 1);
                indices.push(index + 2);
                indices.push(index + 2);
                indices.push(index + 3);
                indices.push(index);

                index += 4;

                // Front
                positions.push([x - 0.5, y - 0.5, z + 0.5]);
                positions.push([x + 0.5, y - 0.5, z + 0.5]);
                positions.push([x + 0.5, y + 0.5, z + 0.5]);
                positions.push([x - 0.5, y + 0.5, z + 0.5]);

                normals.push([0.0, 0.0, 1.0]);
                normals.push([0.0, 0.0, 1.0]);
                normals.push([0.0, 0.0, 1.0]);
                normals.push([0.0, 0.0, 1.0]);

                texcoords.push([0.0, 0.0]);
                texcoords.push([0.0, 1.0]);
                texcoords.push([1.0, 1.0]);
                texcoords.push([1.0, 0.0]);

                indices.push(index);
                indices.push(index + 1);
                indices.push(index + 2);
                indices.push(index + 2);
                indices.push(index + 3);
                indices.push(index);

                index += 4;

                // Back
                positions.push([x - 0.5, y - 0.5, z - 0.5]);
                positions.push([x + 0.5, y - 0.5, z - 0.5]);
                positions.push([x + 0.5, y + 0.5, z - 0.5]);
                positions.push([x - 0.5, y + 0.5, z - 0.5]);

                normals.push([0.0, 0.0, -1.0]);
                normals.push([0.0, 0.0, -1.0]);
                normals.push([0.0, 0.0, -1.0]);
                normals.push([0.0, 0.0, -1.0]);

                texcoords.push([0.0, 0.0]);
                texcoords.push([0.0, 1.0]);
                texcoords.push([1.0, 1.0]);
                texcoords.push([1.0, 0.0]);

                indices.push(index);
                indices.push(index + 3);
                indices.push(index + 2);
                indices.push(index + 2);
                indices.push(index + 1);
                indices.push(index);
            }
        }
    }

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, texcoords);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.set_indices(Some(mesh::Indices::U32(indices)));

    mesh
}
