use java_rand::Random;
use vocs::indexed::Target;
use vocs::view::QuadMut;
use vocs::position::QuadPosition;
use decorator::{Decorator, Result};
use matcher::BlockMatcher;

pub struct ReedDecorator<B, M, L, R> where B: Target, M: BlockMatcher<B>, L: BlockMatcher<B>, R: BlockMatcher<B> {
	pub block: B,
	pub base: M,
	pub liquid: L,
	pub replace: R,
	pub base_height: u8,
	pub add_height: u8
}

impl<B, M, L, R> Decorator<B> for PlantDecorator<B, M, L, R> where B: Target, M: BlockMatcher<B>, L: BlockMatcher<B>, R: BlockMatcher<B> {
	fn generate(&self, quad: &mut QuadMut<B>, rng: &mut Random, position: QuadPosition) -> Result {
		if let Some(candidate) = quad.get(position) {
			if !self.replace.matches(candidate) {
				return Ok(());
			}
		}

		let mut adjacent = 0;

		if let Some(candidate) = quad.offset(-1, -1, 0).and_then(|at| quad.get(at)) {
			if self.liquid.matches(candidate) {
				adjacent += 1;
			}
		}

		if let Some(candidate) = quad.offset(1, -1, 0).and_then(|at| quad.get(at)) {
			if self.liquid.matches(candidate) {
				adjacent += 1;
			}
		}

		if let Some(candidate) = quad.offset(0, -1, -1).and_then(|at| quad.get(at)) {
			if self.liquid.matches(candidate) {
				adjacent += 1;
			}
		}

		if let Some(candidate) = quad.offset(0, -1, 1).and_then(|at| quad.get(at)) {
			if self.liquid.matches(candidate) {
				adjacent += 1;
			}
		}

		if adjacent == 0 {
			return Ok(());
		}

		let height = rng.next_i32(self.add_height as i32 + 1);
		let height = (self.base_height as i32 + rng.next_i32(height + 1)) as i8;

		match position.offset(0, -1, 0) {
			Some(below) => match quad.get(below) {
				Some(candidate) => if !self.base.matches(candidate) {
					return Ok(())
				},
				None => return Ok(())
			},
			None => return Ok(())
		}

		for y in 0..height {
			if let Some(at) = position.offset(0, 1, 0) {
				if let Some(candidate) = quad.get(at) {
					if !self.replace.matches(candidate) {
						return Ok(());
					}
				}

				quad.set_immediate(at, &self.block);
			}
		}

		Ok(())
	}
}