use super::Perlin;

pub struct FractalNoise {
    perlin: Perlin,

    octaves: u32,
    lacunarity: f64,
    persistance: f64,
}

impl FractalNoise {
    pub fn new(seed: u64, octaves: u32, lacunarity: f64, persistance: f64) -> Self {
        let perlin = Perlin::new(seed);

        FractalNoise {
            perlin,

            octaves,
            lacunarity,
            persistance,
        }
    }

    pub fn sample(&self, x: f64, y: f64) -> f64 {
        let mut total = 0.0;
        let mut amplitude = 1.0;
        let mut frequency = 1.0;
        let mut max_val = 0.0;

        for _ in 0..self.octaves {
            total += self.perlin.sample(x * frequency, y * frequency) * amplitude;
            max_val += amplitude;
            amplitude *= self.persistance;
            frequency *= self.lacunarity;
        }

        total / max_val
    }
}
