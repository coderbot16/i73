// Note: The values returned by these functions may be off by up to ±0.00000000000001 units compared to Notchian implementations due to moving around FP operations.
// The simplex implementation may also suffer from some inaccuracy due to it doing the *70 and *amplitude multiplications seperately.

use nalgebra::{Vector2, Vector3};
use sample::{Sample, Layer};
use noise::simplex::Simplex;
use noise::perlin::Perlin;
use rng::JavaRng;

#[derive(Debug)]
pub struct SimplexOctaves(Vec<Simplex>);
impl SimplexOctaves {
	pub fn new(rng: &mut JavaRng, octaves: usize, fq: f64, persistence: f64, scale: (f64, f64)) -> Self {
		let mut octaves = Vec::with_capacity(octaves);
		
		let scale = (scale.0 / 1.5, scale.1 / 1.5);
		let mut frequency = 1.0;
		let mut amplitude = 1.0;
		
		for _ in 0..octaves.capacity() {
			octaves.push(Simplex::from_rng(rng, Vector2::new(scale.0 * frequency, scale.1 * frequency),  0.55 / amplitude));
			
			frequency *= fq;
			amplitude *= persistence;
		}
		
		SimplexOctaves(octaves)
	}
}

impl Sample for SimplexOctaves {
	type Output = f64;
	
	fn sample(&self, point: Vector2<f64>) -> Self::Output {
		let mut result = 0.0;
		
		for octave in &self.0 {
			result += octave.sample(point)
		}
		
		result
	}
	
	fn chunk(&self, chunk: (f64, f64)) -> Layer<Self::Output> {
		let mut result = Layer::fill(0.0);
		
		for octave in &self.0 {
			result += octave.chunk(chunk);
		}
		
		result
	}
}

pub struct PerlinOctaves(Vec<(Perlin, Vec<f64>)>);
impl PerlinOctaves {
	pub fn new(rng: &mut JavaRng, octaves: usize, scale: Vector3<f64>, start: f64, count: usize) -> Self {
		let mut octaves = Vec::with_capacity(octaves);
		
		let mut frequency = 1.0;
		
		for _ in 0..octaves.capacity() {
			let perlin = Perlin::from_rng(rng, scale * frequency, 1.0 / frequency);
			let table = perlin.generate_y_table(start, count);
			
			octaves.push((perlin, table));
			
			frequency /= 2.0;
		}
		
		PerlinOctaves(octaves)
	}
	
	pub fn generate_override(&self, point: Vector3<f64>, index: usize) -> f64 {
		let mut result = 0.0;
		
		for octave in &self.0 {
			result += octave.0.generate_override(point, octave.1[index])
		}
		
		result
	}
}

impl Sample for PerlinOctaves {
	type Output = f64;
	
	fn sample(&self, point: Vector2<f64>) -> Self::Output {
		let mut result = 0.0;
		
		for octave in &self.0 {
			result += octave.0.sample(point)
		}
		
		result
	}
	
	fn chunk(&self, chunk: (f64, f64)) -> Layer<Self::Output> {
		let mut result = Layer::fill(0.0);
		
		for octave in &self.0 {
			result += octave.0.chunk(chunk);
		}
		
		result
	}
}