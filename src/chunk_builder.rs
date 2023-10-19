use bevy::{
    prelude::Mesh,
    render::{mesh, render_resource::PrimitiveTopology},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Index(u32);

#[derive(Default)]
pub struct ChunkBuilder {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    texcoords: Vec<[f32; 2]>,
    indices: Vec<u32>,
}

impl ChunkBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn vertex(&mut self, position: [f32; 3], normal: [f32; 3], texcoord: [f32; 2]) -> Index {
        let index = self.positions.len();
        self.positions.push(position);
        self.normals.push(normal);
        self.texcoords.push(texcoord);
        Index(index as u32)
    }

    pub fn indices(&mut self, index: impl IntoIterator<Item = Index>) {
        self.indices.extend(index.into_iter().map(|index| index.0));
    }

    pub fn build(self) -> Mesh {
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, self.positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, self.texcoords);
        mesh.set_indices(Some(mesh::Indices::U32(self.indices)));
        mesh
    }
}
