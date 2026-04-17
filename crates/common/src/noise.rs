//! Seeded 2D Perlin noise and fractional Brownian motion (FBM).
//!
//! This is a small, self-contained replacement for the subset of the
//! `noise` crate that worldgen actually uses: Perlin with a `u32`
//! seed, FBM over Perlin with configurable octaves and persistence, and a
//! `get([x, y])` sampling API.

use std::f64::consts::PI;

#[derive(Debug, Clone)]
pub struct Perlin {
    permutation: [u8; 512],
}

impl Perlin {
    pub fn new(seed: u32) -> Self {
        let mut perm = [0u8; 256];
        for (i, slot) in perm.iter_mut().enumerate() {
            *slot = i as u8;
        }

        // Expand the 32-bit seed with a constant so seed 0 doesn't produce a
        // degenerate shuffle on the first step.
        let mut state = (seed as u64) ^ 0x9E37_79B9_7F4A_7C15;
        for i in (1..256).rev() {
            let j = (splitmix64(&mut state) as usize) % (i + 1);
            perm.swap(i, j);
        }

        let mut permutation = [0u8; 512];
        for i in 0..512 {
            permutation[i] = perm[i & 255];
        }
        Self { permutation }
    }

    pub fn get(&self, point: [f64; 2]) -> f64 {
        let [x, y] = point;

        let xi = (x.floor() as i64).rem_euclid(256) as usize;
        let yi = (y.floor() as i64).rem_euclid(256) as usize;

        let xf = x - x.floor();
        let yf = y - y.floor();

        let u = fade(xf);
        let v = fade(yf);

        let p = &self.permutation;
        let a = p[xi] as usize + yi;
        let b = p[xi + 1] as usize + yi;

        let x1 = lerp(u, grad(p[a], xf, yf), grad(p[b], xf - 1.0, yf));
        let x2 = lerp(
            u,
            grad(p[a + 1], xf, yf - 1.0),
            grad(p[b + 1], xf - 1.0, yf - 1.0),
        );

        // Raw 2D Perlin peaks around ±√½; scale so output is roughly in [-1, 1].
        lerp(v, x1, x2) * std::f64::consts::SQRT_2
    }
}

#[derive(Debug, Clone)]
pub struct Fbm {
    seed: u32,
    sources: Vec<Perlin>,
    pub octaves: usize,
    pub frequency: f64,
    pub lacunarity: f64,
    pub persistence: f64,
}

impl Fbm {
    pub const DEFAULT_OCTAVES: usize = 6;
    pub const DEFAULT_FREQUENCY: f64 = 1.0;
    /// Matches the `noise` crate's default: an irrational lacunarity avoids
    /// octaves lining up on a repeating lattice.
    pub const DEFAULT_LACUNARITY: f64 = PI * 2.0 / 3.0;
    pub const DEFAULT_PERSISTENCE: f64 = 0.5;
    pub const MAX_OCTAVES: usize = 32;

    pub fn new(seed: u32) -> Self {
        Self {
            seed,
            sources: build_sources(seed, Self::DEFAULT_OCTAVES),
            octaves: Self::DEFAULT_OCTAVES,
            frequency: Self::DEFAULT_FREQUENCY,
            lacunarity: Self::DEFAULT_LACUNARITY,
            persistence: Self::DEFAULT_PERSISTENCE,
        }
    }

    #[must_use]
    pub fn set_octaves(mut self, octaves: usize) -> Self {
        let octaves = octaves.clamp(1, Self::MAX_OCTAVES);
        if octaves != self.octaves {
            self.sources = build_sources(self.seed, octaves);
            self.octaves = octaves;
        }
        self
    }

    #[must_use]
    pub const fn set_persistence(mut self, persistence: f64) -> Self {
        self.persistence = persistence;
        self
    }

    pub fn get(&self, point: [f64; 2]) -> f64 {
        let mut result = 0.0;
        let mut frequency = self.frequency;
        let mut amplitude = 1.0;
        for source in &self.sources {
            let p = [point[0] * frequency, point[1] * frequency];
            result += source.get(p) * amplitude;
            frequency *= self.lacunarity;
            amplitude *= self.persistence;
        }
        result
    }
}

fn build_sources(seed: u32, octaves: usize) -> Vec<Perlin> {
    (0..octaves)
        .map(|i| Perlin::new(seed.wrapping_add(i as u32)))
        .collect()
}

fn fade(t: f64) -> f64 {
    // 6t^5 - 15t^4 + 10t^3
    t * t * t * t.mul_add(t.mul_add(6.0, -15.0), 10.0)
}

fn lerp(t: f64, a: f64, b: f64) -> f64 {
    t.mul_add(b - a, a)
}

fn grad(hash: u8, x: f64, y: f64) -> f64 {
    match hash & 7 {
        0 => x + y,
        1 => -x + y,
        2 => x - y,
        3 => -x - y,
        4 => x,
        5 => -x,
        6 => y,
        _ => -y,
    }
}

const fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn perlin_is_zero_at_integer_lattice_points() {
        let p = Perlin::new(42);
        for x in -3..=3 {
            for y in -3..=3 {
                let v = p.get([x as f64, y as f64]);
                assert!(v.abs() < 1e-9, "expected 0 at ({x}, {y}), got {v}");
            }
        }
    }

    #[test]
    fn perlin_is_deterministic() {
        let a = Perlin::new(1234);
        let b = Perlin::new(1234);
        for i in 0..50 {
            let p = [i as f64 * 0.37, i as f64 * 0.91];
            assert_eq!(a.get(p), b.get(p));
        }
    }

    #[test]
    fn perlin_different_seeds_differ() {
        let a = Perlin::new(1);
        let b = Perlin::new(2);
        let differs = (0..100)
            .map(|i| [(i as f64).mul_add(0.1, 0.37), (i as f64).mul_add(0.1, 0.91)])
            .any(|p| a.get(p) != b.get(p));
        assert!(differs);
    }

    #[test]
    fn perlin_output_roughly_bounded() {
        let p = Perlin::new(7);
        let mut max_abs = 0.0f64;
        for i in 0..200 {
            for j in 0..200 {
                let v = p.get([i as f64 * 0.13, j as f64 * 0.17]);
                max_abs = max_abs.max(v.abs());
            }
        }
        // Scaled 2D Perlin occasionally exceeds 1 but should not blow up.
        assert!(max_abs < 1.5, "max |Perlin| = {max_abs} is too large");
    }

    #[test]
    fn fbm_respects_octaves_and_persistence() {
        let base = Fbm::new(1);
        let tuned = Fbm::new(1).set_octaves(8).set_persistence(0.7);
        assert_eq!(base.octaves, Fbm::DEFAULT_OCTAVES);
        assert_eq!(tuned.octaves, 8);
        assert_eq!(tuned.persistence, 0.7);
        // Different parameters should produce different samples.
        assert_ne!(base.get([0.3, 0.4]), tuned.get([0.3, 0.4]));
    }
}
