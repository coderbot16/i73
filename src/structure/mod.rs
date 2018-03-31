pub mod caves;
pub mod organized;

use rng::JavaRng;
use vocs::world::view::ColumnMut;
use vocs::world::chunk::Target;
use std::marker::PhantomData;
use generator::Pass;

pub struct StructureGenerateNearby<T, B> where T: StructureGenerator<B>, B: Target {
	seed_coefficients: (i64, i64),
	radius: i32,
	diameter: i32,
	world_seed: i64,
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
			world_seed,
			generator,
			phantom: PhantomData
		}
	}
}

impl<T, B> Pass<B> for StructureGenerateNearby<T, B> where T: StructureGenerator<B>, B: Target {
	fn apply(&self, target: &mut ColumnMut<B>, chunk: (i32, i32)) {
		for x in     (0..self.diameter).map(|x| chunk.0 + x - self.radius) {
			for z in (0..self.diameter).map(|z| chunk.1 + z - self.radius) {
				let x_part = (x as i64).wrapping_mul(self.seed_coefficients.0);
				let z_part = (z as i64).wrapping_mul(self.seed_coefficients.1);
				
				let seed = (x_part.wrapping_add(z_part)) ^ self.world_seed;
				
				self.generator.generate(JavaRng::new(seed), target, chunk, (x, z), self.radius);
			}
		}
	}
}

pub trait StructureGenerator<B> where B: Target {
	fn generate(&self, random: JavaRng, column: &mut ColumnMut<B>, chunk_pos: (i32, i32), from: (i32, i32), radius: i32);
}