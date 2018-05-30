use java_rand::Random;
use vocs::indexed::Target;
use vocs::view::QuadMut;
use vocs::position::{QuadPosition, Offset, dir};
use decorator::{Decorator, Result};
use matcher::BlockMatcher;

// Pumpkin: On grass, replacing air or {material:ground_cover}

pub struct PlantDecorator<B> where B: Target {
	pub block: B,
	pub base: BlockMatcher<B>,
	pub replace: BlockMatcher<B>
}

impl<B> Decorator<B> for PlantDecorator<B> where B: Target {
	fn generate(&self, quad: &mut QuadMut<B>, _: &mut Random, position: QuadPosition) -> Result {
		// TODO: Check if the block is above the heightmap (how?)

		if !self.replace.matches(quad.get(position)) {
			return Ok(());
		}

		match position.offset(dir::Down) {
			Some(below) => if !self.base.matches(quad.get(below)) {
				return Ok(())
			},
			None => return Ok(())
		}

		quad.set_immediate(position, &self.block);

		Ok(())
	}
}