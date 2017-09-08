pub mod perlin;
pub mod simplex;
pub mod octaves;

use rng::JavaRng;
use nalgebra::Vector3;

pub struct Permutations {
	offset: Vector3<f64>,
	permutations: [u8; 256]
}

impl Permutations {
	pub fn new(rng: &mut JavaRng) -> Self {
		let mut p = Permutations {
			offset: Vector3::new(rng.next_f64() * 256.0, rng.next_f64() * 256.0, rng.next_f64() * 256.0),
			permutations: [0; 256]
		};
		
		// Fill array with 0..256
		for (i, x) in p.permutations.iter_mut().enumerate() {
			*x = i as u8;
		};
		
		for i in 0..256 {
			let rand = rng.next_i32(256 - i) + i;
			p.permutations.swap(i as usize, rand as usize);
		};
		
		p
	}
	
	fn hash(&self, i: u16) -> u16 {
		self.permutations[(i as usize) & 0xFF] as u16
	}
}

impl ::std::fmt::Debug for Permutations {
	fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
		write!(f, "Permutations {{ offset: ({}, {}, {}), permutations: {:?} }}", self.offset.x, self.offset.y, self.offset.z, &self.permutations[..])
	}
}