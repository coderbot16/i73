use std::ops::{Add, AddAssign};
use nalgebra::Vector2;

pub struct Layer<T>(pub [T; 256]) where T: Copy;
impl<T> Layer<T> where T: Copy {
	pub fn fill(fill: T) -> Self {
		Layer([fill; 256])
	}
	
	pub fn get(&self, x: usize, z: usize) -> T {
		self.0[x*16 + z]
	}
	
	fn set(&mut self, x: usize, z: usize, value: T) {
		self.0[x*16 + z] = value;
	}
}

impl<T> Add for Layer<T> where T: Copy + AddAssign {
	type Output = Self;
	
	fn add(mut self, rhs: Self) -> Self::Output {
		for x in 0..256 {
			// TODO: Iterators.
			self.0[x] += rhs.0[x];
		}
		
		self
	}
}

impl<T> AddAssign for Layer<T> where T: Copy + AddAssign {
	fn add_assign(&mut self, rhs: Self) {
		for x in 0..256 {
			// TODO: Iterators.
			self.0[x] += rhs.0[x];
		}
	}
}

pub trait Sample {
	type Output: Default + Copy;
	
	/// Coordinates are in block space
	fn sample(&self, point: Vector2<f64>) -> Self::Output;
	
	/// An optimized version of this function is usually provided by the implementor.
	fn chunk(&self, chunk: (f64, f64)) -> Layer<Self::Output> {
		let mut out = Layer::fill(Self::Output::default());
		
		for x in 0..16 {
			let cx = chunk.0 + (x as f64);
			
			for z in 0..16 {
				let cz = chunk.1 + (z as f64);
				
				out.set(x, z, self.sample(Vector2::new(cx, cz)));
			}
		}
		
		out
	}
}