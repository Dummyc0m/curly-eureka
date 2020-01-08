use ndarray::{Array3, Shape, Array2};
use simdnoise::NoiseBuilder;

pub const WIDTH: usize = 64;
pub const DEPTH: usize = 64;
pub const HEIGHT: usize = 256;

#[derive(Clone, Debug)]
pub struct Chunk {
  x: i32, z: i32,
  data: Array3<f32>,
}

impl Chunk {
  pub fn get(&self, idx: (usize, usize, usize)) -> Option<f32> {
    self.data.get(idx).copied()
  }

  pub fn iter(&self) -> impl Iterator<Item = ((usize, usize, usize), &f32)> {
    self.data.indexed_iter()
  }
}

pub struct TerrainGenerator {
  seed: i32,
}

impl TerrainGenerator {
  pub fn new(seed: i32) -> TerrainGenerator {
    TerrainGenerator { seed }
  }

  pub fn generate_chunk(&self, x: i32, z: i32) -> Chunk {
    let (data, min, max) =
      NoiseBuilder::ridge_2d_offset((x * WIDTH as i32) as f32, WIDTH, (z * DEPTH as i32) as f32, DEPTH)
      .with_seed(self.seed)
      .with_lacunarity(0.5)
      .with_freq(0.04)
      .with_gain(4.0)
      .with_octaves(4)
      .generate()
      ;
    let data = data.into_iter().map(|v| v * 25.0).collect::<Vec<_>>();
    let height_map = Array2::from_shape_vec((WIDTH, DEPTH), data).unwrap();
    let data = Array3::from_shape_fn((WIDTH, HEIGHT, DEPTH), |(x, y, z)| {
      let height: f32 = *height_map.get((x, z)).unwrap();
      if height.floor() as usize > y {
        1.0
      } else {
        let cell_value = height - (y as f32);
        cell_value
      }
    });

    Chunk {
      x, z, data
    }
  }
}

