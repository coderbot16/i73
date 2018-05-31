use std::ops::{Add, AddAssign};
use cgmath::{Point2, Vector2};
use vocs::position::LayerPosition;

pub struct Layer<T>([T; 256]) where T: Copy;
impl<T> Layer<T> where T: Copy {
	pub fn fill(fill: T) -> Self {
		Layer([fill; 256])
	}
	
	pub fn get(&self, position: LayerPosition) -> T {
		self.0[position.zx() as usize]
	}
	
	fn set(&mut self, position: LayerPosition, value: T) {
		self.0[position.zx() as usize] = value;
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
	fn sample(&self, point: Point2<f64>) -> Self::Output;
	
	/// An optimized version of this function is usually provided by the implementor.
	fn chunk(&self, chunk: (f64, f64)) -> Layer<Self::Output> {
		let mut out = Layer::fill(Self::Output::default());
		let chunk = Point2::new(chunk.0, chunk.1);

		for index in 0..=255 {
			let position = LayerPosition::from_zx(index);
			let point = chunk + Vector2::new(position.x() as f64, position.z() as f64);

			out.set(position, self.sample(point));
		}
		
		out
	}
}