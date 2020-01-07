use amethyst::{
  renderer::{
    rendy::mesh::{Color, MeshBuilder, Normal, Position, Tangent, TexCoord},
  },
  core::math::Vector3
};
use num_traits::zero;

pub fn calculate_normals(positions: &[Position], indices: &[u16]) -> Vec<Normal> {
  let mut normals = vec![zero::<Vector3<f32>>(); positions.len()];
  let num_faces = indices.len() / 3;
  for face in 0..num_faces {
    let i0 = indices[face * 3 + 0] as usize;
    let i1 = indices[face * 3 + 1] as usize;
    let i2 = indices[face * 3 + 2] as usize;
    let a = Vector3::from(positions[i0].0);
    let b = Vector3::from(positions[i1].0);
    let c = Vector3::from(positions[i2].0);
    let n = (b - a).cross(&(c - a));
    normals[i0] += n;
    normals[i1] += n;
    normals[i2] += n;
  }
  normals
    .into_iter()
    .map(|n| Normal(n.normalize().into()))
    .collect::<Vec<_>>()
}