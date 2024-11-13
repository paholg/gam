use std::f32::consts::PI;

use bevy::prelude::Mesh;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::PrimitiveTopology;

/// A hollow polygon. Can also act as a hollow circle when vertices is large.
pub struct HollowPolygon {
    pub radius: f32,
    pub thickness: f32,
    pub vertices: u32,
}

impl From<HollowPolygon> for Mesh {
    fn from(value: HollowPolygon) -> Self {
        let HollowPolygon {
            radius,
            thickness,
            vertices,
        } = value;

        let chunk = Chunk {
            radius,
            inner_radius: radius - thickness,
            vertices,
            angle: 2.0 * PI,
        };

        chunk.into()
    }
}

/// A shape similar to a 2d pineapple chunk. It is a section of hollow polygon
/// centered on the y-axis. If the angle is 2*pi, then it will be the same as a
/// [`HollowPolygon`].
pub struct Chunk {
    pub radius: f32,
    pub inner_radius: f32,
    pub vertices: u32,
    pub angle: f32,
}

impl Chunk {
    fn into_positions_and_indices(self) -> (Vec<[f32; 3]>, Vec<u32>) {
        let Chunk {
            radius,
            inner_radius,
            vertices,
            angle,
        } = self;

        assert!(vertices >= 2);

        let mut positions = Vec::with_capacity(vertices as usize * 2);
        let mut indices = Vec::with_capacity(vertices as usize * 2 * 3);

        for r in [inner_radius, radius] {
            for i in 0..vertices {
                let phi = angle / vertices as f32 * i as f32 - angle / 2.0 + PI * 0.5;
                let coord = [r * phi.cos(), r * phi.sin(), 0.0];
                positions.push(coord);
            }
        }

        for i in 0..vertices {
            let n = vertices + i;
            let n_1 = if i + 1 >= vertices {
                i + 1
            } else {
                vertices + i + 1
            };
            let i_1 = if i + 1 >= vertices {
                i + 1 - vertices
            } else {
                i + 1
            };
            indices.extend_from_slice(&[i, n, n_1]);
            indices.extend_from_slice(&[i_1, i, n_1]);
        }
        (positions, indices)
    }
}

impl From<Chunk> for Mesh {
    fn from(value: Chunk) -> Self {
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::RENDER_WORLD,
        );
        let (positions, indices) = value.into_positions_and_indices();

        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_indices(bevy::render::mesh::Indices::U32(indices));

        mesh
    }
}

/// See [`Chunk`]. This is a hollow version.
pub struct HollowChunk {
    pub radius: f32,
    pub inner_radius: f32,
    pub thickness: f32,
    pub vertices: u32,
    pub angle: f32,
}
impl From<HollowChunk> for Mesh {
    fn from(value: HollowChunk) -> Self {
        let HollowChunk {
            radius,
            inner_radius,
            thickness,
            vertices,
            angle,
        } = value;
        // Let's start by making Chunks of our outer and inner edges.
        let (outer_positions, outer_indices) = Chunk {
            radius,
            inner_radius: radius - thickness,
            vertices,
            angle,
        }
        .into_positions_and_indices();
        let (inner_positions, inner_indices) = Chunk {
            radius: inner_radius + thickness,
            inner_radius,
            vertices,
            angle,
        }
        .into_positions_and_indices();
        let len = outer_positions.len() as u32;
        let mut positions = outer_positions;
        positions.extend_from_slice(&inner_positions);
        // Before we combine the indices, we need to fix them up.
        let mut indices = outer_indices;
        indices.extend(inner_indices.into_iter().map(|idx| idx + len));

        // Now we need two final vertices to complete our sides.
        // FIXME: Do this.

        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::RENDER_WORLD,
        );
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_indices(bevy::render::mesh::Indices::U32(indices));

        mesh
    }
}
