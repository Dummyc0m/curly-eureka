use ndarray::Array3;
use amethyst::{shrev::EventChannel, core::math::{try_convert, Vector3}, renderer::types::MeshData};
use derive_more::{From, Into};
use std::convert;
use super::terrain_gen::TerrainGenerator;
use dashmap::DashMap;

// The idea is to have chunk manager load chunks via ChunkSystem
// Then we regenerate the meshes as required
#[derive(From, Into, Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct VoxelPos(Vector3<i32>);

#[derive(From, Into, Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct ChunkPos(Vector3<i32>);

#[derive(From, Into, Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct LocalPos(Vector3<u8>);

impl VoxelPos {
  pub fn as_local(self, pos: ChunkPos) -> LocalPos {
    // unwrap should succeed because the value ought to fit
    LocalPos(try_convert(self.0 - VoxelPos::from(pos).0).unwrap())
  }
}

impl LocalPos {
  pub fn as_voxel(self, pos: ChunkPos) -> VoxelPos {
    let local_pos : Vector3<i32> = convert(self.0);
    VoxelPos(local_pos + VoxelPos::from(pos).0)
  }
}

impl convert::From<ChunkPos> for VoxelPos {
  fn from(pos: ChunkPos) -> Self {
    VoxelPos(pos.0 * CHUNK_LEN_I32)
  }
}

impl convert::From<VoxelPos> for ChunkPos {
  fn from(pos: VoxelPos) -> Self {
    ChunkPos(pos.0 / CHUNK_LEN_I32)
  }
}

#[derive(Clone, Debug)]
pub struct Chunk {
  pos: ChunkPos,
  data: Array3<f32>,
}

impl Chunk {
  pub fn new(pos: ChunkPos, data: Array3<f32>) -> Self { Self { pos, data } }

  pub fn get(&self, idx: VoxelPos) -> Option<f32> {
    let idx: Vector3<usize> = convert(idx.as_local(self.pos).0);
    self.data.get(idx.into()).copied()
  }

  pub fn iter(&self) -> impl Iterator<Item = (VoxelPos, f32)> {
    self.data.indexed_iter().map(|((x, y, z), v)| {
      let loc: Vector3<i32> = convert(Vector3::new(x, y, z));
      (VoxelPos::from(loc), *v)
    })
  }
}

pub struct ChunkState {
  chunk: Chunk,
  mesh: MeshData,
}

pub enum ChunkEvent {
  Write(Vector3<i32>, f32)
}

pub struct ChunkManager {
  loaded: DashMap<ChunkPos, ChunkState>,
  terrain_gen: TerrainGenerator,
  surface_net: SurfaceNet,
}

impl ChunkManager {
  pub fn new(terrain_gen: TerrainGenerator, surface_net: SurfaceNet) -> ChunkManager {
    ChunkManager {
      loaded: DashMap::new(),
      terrain_gen,
      surface_net,
    }
  }

  pub fn load_chunk(&self, chunk_pos: ChunkPos) {
  }

  pub fn get_chunk(&self, chunk_pos: ChunkPos) -> Option<ElementGuard<ChunkState>> {
    self.loaded.get(&chunk_pos)
  }
}
