use ndarray::{Array3, Shape, Array2};
use simdnoise::NoiseBuilder;
use crate::config::generator::GeneratorConfig;

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
  config: GeneratorConfig,
}

impl TerrainGenerator {
  pub fn new(config: GeneratorConfig) -> TerrainGenerator {
    TerrainGenerator { config }
  }

  pub fn generate_chunk(&self, x: i32, z: i32) -> Chunk {
    let (data, min, max) =
      // NoiseBuilder::ridge_2d_offset((x * WIDTH as i32) as f32, WIDTH + 1, (x * DEPTH as i32) as f32, DEPTH + 1)
      NoiseBuilder::ridge_2d_offset((64 * x) as f32, WIDTH + 3, (64 * z) as f32, DEPTH + 3)
      .with_seed(self.config.seed)
      .with_lacunarity(self.config.lacunarity)
      .with_freq(self.config.freq)
      .with_gain(self.config.gain)
      .with_octaves(self.config.octaves)
      .generate();
    let data = data.into_iter().map(|v| v * self.config.scaling).collect::<Vec<_>>();
    let height_map = Array2::from_shape_vec((DEPTH + 3, WIDTH + 3), data).unwrap();
    let data = Array3::from_shape_fn((WIDTH + 3, HEIGHT + 3, DEPTH + 3), |(x, y, z)| {
      let height: f32 = *height_map.get((z, x)).unwrap();
      if height.floor() as usize > y {
        1.0
      } else {
        // sharp vs smooth
        if self.config.cutoff {
          0.0
        } else {
          height - (y as f32)
        }
      }
    });

    Chunk {
      x, z, data
    }
  }
}

