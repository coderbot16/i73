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

#[derive(Debug)]
pub struct PerlinOctaves(Vec<Perlin>);
impl PerlinOctaves {
	pub fn new(rng: &mut JavaRng, octaves: usize, scale: Vector3<f64>) -> Self {
		let mut octaves = Vec::with_capacity(octaves);
		
		let mut frequency = 1.0;
		
		for _ in 0..octaves.capacity() {
			octaves.push(Perlin::from_rng(rng, scale * frequency, 1.0 / frequency));
			
			frequency /= 2.0;
		}
		
		PerlinOctaves(octaves)
	}
	
	fn generate_y_tables(&self, y_start: f64, y_count: usize) -> Vec<f64> {
		let mut tables = vec![0.0; y_count * self.0.len()];
		
		for (perlin, chunk) in self.0.iter().zip(tables.chunks_mut(y_count)) {
			perlin.generate_y_table(y_start, chunk);
		}
		
		tables
	}
	
	pub fn vertical_ref(&self, y_start: f64, y_count: usize) -> PerlinOctavesVerticalRef {
		PerlinOctavesVerticalRef {
			octaves: &self.0,
			tables: self.generate_y_tables(y_start, y_count),
			y_count
		}
	}
	
	pub fn into_vertical(self, y_start: f64, y_count: usize) -> PerlinOctavesVertical {
		let tables = self.generate_y_tables(y_start, y_count);
		
		PerlinOctavesVertical { octaves: self.0, tables, y_count }
	}
	
	pub fn generate(&self, point: Vector3<f64>) -> f64 {
		self.0.iter().fold(0.0, |result, perlin| result + perlin.generate(point))
	}
}

impl Sample for PerlinOctaves {
	type Output = f64;
	
	fn sample(&self, point: Vector2<f64>) -> Self::Output {
		self.0.iter().fold(0.0, |result, perlin| result + perlin.sample(point))
	}
	
	fn chunk(&self, chunk: (f64, f64)) -> Layer<Self::Output> {
		self.0.iter().fold(Layer::fill(0.0), |result, perlin| result + perlin.chunk(chunk))
	}
}	

pub struct PerlinOctavesVerticalRef<'a> {
	octaves: &'a [Perlin],
	tables:  Vec<f64>,
	y_count: usize
}

impl<'a> PerlinOctavesVerticalRef<'a> {
	pub fn generate_override(&self, point: Vector3<f64>, index: usize) -> f64 {
		let mut result = 0.0;
		
		for (perlin, table) in self.octaves.iter().zip(self.tables.chunks(self.y_count)) {
			result += perlin.generate_override(point, table[index])
		}
		
		result
	}
}


pub struct PerlinOctavesVertical {
	octaves: Vec<Perlin>,
	tables:  Vec<f64>,
	y_count: usize
}

impl PerlinOctavesVertical {
	pub fn new(rng: &mut JavaRng, octaves: usize, scale: Vector3<f64>, y_start: f64, y_count: usize) -> Self {
		PerlinOctaves::new(rng, octaves, scale).into_vertical(y_start, y_count)
	}
	
	pub fn generate_override(&self, point: Vector3<f64>, index: usize) -> f64 {
		let mut result = 0.0;
		
		for (perlin, table) in self.octaves.iter().zip(self.tables.chunks(self.y_count)) {
			result += perlin.generate_override(point, table[index])
		}
		
		result
	}
}