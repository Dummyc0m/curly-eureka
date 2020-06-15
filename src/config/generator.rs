use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GeneratorConfig {
    pub seed: i32,
    pub lacunarity: f32,
    pub freq: f32,
    pub gain: f32,
    pub octaves: u8,

    pub scaling: f32,
    pub cutoff: bool,
}

impl Default for GeneratorConfig {
    fn default() -> Self {
        GeneratorConfig {
            seed: 25565,
            lacunarity: 0.5,
            freq: 0.04,
            gain: 4.0,
            octaves: 4,

            scaling: 25.0,
            cutoff: false,
        }
    }
}
