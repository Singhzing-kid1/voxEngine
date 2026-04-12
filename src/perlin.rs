use::std::array;

fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E3779B97F4A7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
    z ^ (z >> 31)
} 
pub struct Perlin {
    perm: [u8; 512],
    seed: u64
}

impl Perlin {
    pub fn new(seed: u64) -> Self {
        let mut perm_init: [u8; 256] = array::from_fn(|i| i as u8);
        let mut state = seed;

        for i in (0..=255).rev() {
            state = splitmix64(&mut state);
            let j = (state % (i + 1) as u64) as usize;
            perm_init.swap(i, j);
        }

        let mut perm = [0u8; 512];

        perm[..256].copy_from_slice(&perm_init);
        perm[256..].copy_from_slice(&perm_init);

        Perlin {
            perm,
            seed
        }
    }

    pub fn sample(&self, x: f64, y: f64) -> f64 {
        let xi = (x.floor() as i32 & 255) as usize;
        let yi = (y.floor() as i32 & 255) as usize;


        let xf = x - x.floor();
        let yf = y - y.floor();

        let u = Self::fade(xf);
        let v = Self::fade(yf);
        

        let aa = self.perm[self.perm[xi] as usize + yi];
        let ab = self.perm[self.perm[xi] as usize + yi + 1];
        let ba = self.perm[self.perm[xi + 1] as usize + yi];
        let bb = self.perm[self.perm[xi + 1] as usize + yi + 1];

        let x1 = Self::lerp(
            Self::grad(aa, xf, yf),
            Self::grad(ba, xf - 1.0, yf),
            u
        );

        let x2 = Self::lerp(
            Self::grad(ab, xf, yf - 1.0),
            Self::grad(bb, xf - 1.0, yf - 1.0),
            u
        );

        Self::lerp(x1, x2, v)
    }
}

impl Perlin {
    fn fade (t: f64) -> f64 {
        t * t * t *(t * (t* 6.0 - 15.0) + 10.0)
    }

    fn lerp(a: f64, b: f64, t: f64) -> f64 {
        a + t *(b - a)
    }

    fn grad(hash: u8, x: f64, y: f64) -> f64 {
        match hash & 3 {
            0 => x + y,
            1 => -x + y,
            2 => x - y,
            3 => -x - y,
            _ => unreachable!()
        }
    }
}


pub struct FractalNoise {
    perlin: Perlin,

    octaves: u32,
    lacunarity: f64,
    persistance: f64
}

impl FractalNoise {
    pub fn new(seed: u64, octaves: u32, lacunarity: f64, persistance: f64) -> Self{
        let perlin = Perlin::new(seed);

        FractalNoise {
            perlin,

            octaves,
            lacunarity,
            persistance
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