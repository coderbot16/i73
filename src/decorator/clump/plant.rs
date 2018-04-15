use rng::JavaRng;
use vocs::indexed::Target;
use vocs::view::QuadMut;
use vocs::position::QuadPosition;
use decorator::{Decorator, Result};
use matcher::BlockMatcher;

pub struct PlantDecorator<B, M, R> where B: Target, M: BlockMatcher<B>, R: BlockMatcher<B> {
	pub block: B,
	pub base: M,
	pub replace: R
}

impl<B, M, R> Decorator<B> for PlantDecorator<B, M, R> where B: Target, M: BlockMatcher<B>, R: BlockMatcher<B> {
	fn generate(&self, quad: &mut QuadMut<B>, _: &mut JavaRng, position: QuadPosition) -> Result {
		// TODO: Check if the block is above the heightmap (how?)

		if let Some(candidate) = quad.get(position) {
			if !self.replace.matches(candidate) {
				return Ok(());
			}
		}

		match position.offset(0, -1, 0) {
			Some(below) => match quad.get(below) {
				Some(candidate) => if !self.base.matches(candidate) {
					return Ok(())
				},
				None => return Ok(())
			},
			None => return Ok(())
		}

		quad.set_immediate(position, &self.block);

		Ok(())
	}
}