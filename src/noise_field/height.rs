use noise::octaves::PerlinOctaves;
use nalgebra::{Vector2, Vector3};
use rng::JavaRng;
use biome::climate::Climate;
use sample::Sample;

#[derive(Debug, Copy, Clone)]
pub struct Height {
	pub center: f64,
	pub chaos:  f64
}

#[derive(Debug)]
pub struct HeightSettings {
	biome_influence_coord_scale: Vector3<f64>,
	biome_influence_scale:       f64,
	depth_coord_scale:           Vector3<f64>,
	depth_scale:                 f64,
	depth_base:                  f64
}

impl Default for HeightSettings {
	fn default() -> Self {
		HeightSettings {
			biome_influence_coord_scale: Vector3::new(1.121, 0.0, 1.121),
			biome_influence_scale:       512.0,
			depth_coord_scale:           Vector3::new(200.0, 0.0, 200.0),
			depth_scale:                 8000.0,
			depth_base:                  8.5
		}
	}
}

pub struct HeightSource {
	biome_influence:       PerlinOctaves,
	depth:                 PerlinOctaves,
	biome_influence_scale: f64,
	depth_scale:           f64,
	depth_base:            f64
}

impl HeightSource {
	pub fn new(rng: &mut JavaRng, settings: &HeightSettings) -> Self {
		HeightSource {
			biome_influence:       PerlinOctaves::new(rng, 10, settings.biome_influence_coord_scale),
			depth:                 PerlinOctaves::new(rng, 16, settings.depth_coord_scale),
			biome_influence_scale: settings.biome_influence_scale,
			depth_scale:           settings.depth_scale,
			depth_base:            settings.depth_base
		}
	}
	
	pub fn sample(&self, point: Vector2<f64>, climate: Climate) -> Height {
		let scaled_noise = self.biome_influence.sample(point) / self.biome_influence_scale;
		
		let chaos = (climate.influence_factor() * (scaled_noise + 0.5)).max(0.0).min(1.0) + 0.5;
		
		let mut depth = self.depth.sample(point) / self.depth_scale;
		
		if depth < 0.0 {
			depth *= 0.3
		}
		
		depth = depth.abs().min(1.0) * 3.0 - 2.0;
		depth /= if depth < 0.0 {1.4} else {2.0};
		
		Height { 
			center: self.depth_base + depth * (self.depth_base / 8.0),
			chaos: if depth < 0.0 {0.5} else {chaos}
		}
	}
}

/// Converts form lerp coords (5x5) to layer coords (16x16).
/// ```
/// 0 => 1
/// 1 => 4
/// 2 => 7
/// 3 => 10
/// 4 => 13
/// ```
pub fn lerp_to_layer(lerp: Vector2<usize>) -> Vector2<usize> {
	lerp.map(|x| x*3 + 1)
}