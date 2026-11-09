use super::helpers;
use std::array;

pub struct Perlin {
    perm: [u8; 512],
    seed: u64,
}

impl Perlin {
    pub fn new(seed: u64) -> Self {
        let mut perm_init: [u8; 256] = array::from_fn(|i| i as u8);
        let mut state = seed;

        for i in (0..=255).rev() {
            state = helpers::splitmix64(&mut state);
            let j = (state % (i + 1) as u64) as usize;
            perm_init.swap(i, j);
        }

        let mut perm = [0u8; 512];

        perm[..256].copy_from_slice(&perm_init);
        perm[256..].copy_from_slice(&perm_init);

        Perlin { perm, seed }
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

        let x1 = Self::lerp(Self::grad(aa, xf, yf), Self::grad(ba, xf - 1.0, yf), u);

        let x2 = Self::lerp(
            Self::grad(ab, xf, yf - 1.0),
            Self::grad(bb, xf - 1.0, yf - 1.0),
            u,
        );

        Self::lerp(x1, x2, v)
    }

    fn fade(t: f64) -> f64 {
        t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
    }

    fn lerp(a: f64, b: f64, t: f64) -> f64 {
        a + t * (b - a)
    }

    fn grad(hash: u8, x: f64, y: f64) -> f64 {
        match hash & 3 {
            0 => x + y,
            1 => -x + y,
            2 => x - y,
            3 => -x - y,
            _ => unreachable!(),
        }
    }
}
