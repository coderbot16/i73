pub mod filter;
pub mod i80;
pub mod zoom;
pub mod blur;

use std::fmt::{Display, Formatter, Result};

#[derive(Debug, Clone)]
pub struct Image<T> {
	data: Box<[T]>,
	size: (usize, usize)
}

impl<T> Image<T> where T: Clone {
	pub fn new(fill: T, size_x: usize, size_z: usize) -> Self {
		Image {
			data: vec![fill; size_x * size_z].into_boxed_slice(),
			size: (size_x, size_z)
		}
	}
}

impl<T> Image<T> {
	pub fn size(&self) -> (usize, usize) {
		self.size
	}
	
	pub fn x_size(&self) -> usize {
		self.size.0
	}
	
	pub fn z_size(&self) -> usize {
		self.size.1
	}
	
	pub fn bounds_check(&self, x: usize, z: usize) {
		if x > self.size.0 {
			panic!("the x size is {} but the x is {}", self.size.0, x);
		}
		
		if z > self.size.1 {
			panic!("the z size is {} but the z is {}", self.size.1, z);
		}
	}
	
	pub fn get(&self, x: usize, z: usize) -> &T {
		self.bounds_check(x, z);
		
		&self.data[z * self.size.0 + x]
	}
	
	pub fn set(&mut self, x: usize, z: usize, value: T) {
		self.bounds_check(x, z);
		
		self.data[z * self.size.0 + x] = value;
	}
}

impl<T> Display for Image<T> where T: Display {
	fn fmt(&self, f: &mut Formatter) -> Result {
		for z in (0..self.z_size()).rev() {
			for x in 0..self.x_size() {
				write!(f, "{} ", self.get(x, z))?;
			}
			
			writeln!(f)?;
		}
		
		Ok(())
	}
}