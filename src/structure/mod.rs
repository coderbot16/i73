pub mod caves;
// TODO: Should mineshafts/villages be implemented?: pub mod organized;

use java_rand::Random;
use vocs::view::ColumnMut;
use vocs::indexed::Target;
use vocs::position::GlobalColumnPosition;
use std::marker::PhantomData;
use generator::Pass;

pub struct StructureGenerateNearby<T, B> where T: StructureGenerator<B>, B: Target {
	seed_coefficients: (i64, i64),
	radius: u32,
	diameter: u32,
	world_seed: u64,
	generator: T,
	phantom: PhantomData<B>
}

impl<T, B> StructureGenerateNearby<T, B> where T: StructureGenerator<B>, B: Target {
	pub fn new(world_seed: u64, radius: u32, generator: T) -> Self {
		let mut rng = Random::new(world_seed);
		
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
	fn apply(&self, target: &mut ColumnMut<B>, chunk: GlobalColumnPosition) {
		let radius = self.radius as i32;

		for x in     (0..self.diameter).map(|x| chunk.x() + (x as i32) - radius) {
			for z in (0..self.diameter).map(|z| chunk.z() + (z as i32) - radius) {
				let x_part = (x as i64).wrapping_mul(self.seed_coefficients.0) as u64;
				let z_part = (z as i64).wrapping_mul(self.seed_coefficients.1) as u64;
				
				let seed = (x_part.wrapping_add(z_part)) ^ self.world_seed;
				let from = GlobalColumnPosition::new(x, z);

				self.generator.generate(Random::new(seed), target, chunk, from, self.radius);
			}
		}
	}
}

pub trait StructureGenerator<B> where B: Target {
	fn generate(&self, random: Random, column: &mut ColumnMut<B>, chunk_pos: GlobalColumnPosition, from: GlobalColumnPosition, radius: u32);
}