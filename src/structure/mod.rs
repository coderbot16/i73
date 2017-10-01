pub mod caves;
use rng::JavaRng;
use chunk::grouping::Column;
use chunk::storage::Target;
use std::marker::PhantomData;

pub struct StructureGenerateNearby<T, B> where T: StructureGenerator<B>, B: Target {
	seed_coefficients: (i64, i64),
	radius: i32,
	diameter: i32,
	generator: T,
	phantom: PhantomData<B>
}

impl<T, B> StructureGenerateNearby<T, B> where T: StructureGenerator<B>, B: Target {
	pub fn new(world_seed: i64, radius: i32, generator: T) -> Self {
		let mut rng = JavaRng::new(world_seed);
		
		StructureGenerateNearby {
			seed_coefficients: (
				((rng.next_i64() >> 1) << 1) + 1, 
				((rng.next_i64() >> 1) << 1) + 1
			),
			radius,
			diameter: radius * 2,
			generator,
			phantom: PhantomData
		}
	}
	
	pub fn generate(&self, column: &mut Column<B>, chunk_pos: (i32, i32)) {
		for x in     (0..self.diameter).map(|x| chunk_pos.0 + x - self.radius) {
			for z in (0..self.diameter).map(|z| chunk_pos.1 + z - self.radius) {
				let seed = (x as i64).wrapping_mul(self.seed_coefficients.0) + (z as i64).wrapping_mul(self.seed_coefficients.1);
				self.generator.generate(JavaRng::new(seed), column, chunk_pos, (x, z));
			}
		}
	}
}

pub trait StructureGenerator<B> where B: Target {
	fn generate(&self, random: JavaRng, column: &mut Column<B>, chunk_pos: (i32, i32), from: (i32, i32));
}