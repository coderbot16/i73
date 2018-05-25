use java_rand::Random;
use vocs::indexed::Target;
use vocs::view::QuadMut;
use vocs::position::{QuadPosition, Offset, dir};
use decorator::{Decorator, Result};
use matcher::BlockMatcher;

pub struct ExposedDecorator<B, S, E> where B: Target, S: BlockMatcher<B>, E: BlockMatcher<B> {
	pub block: B,
	pub stone: S,
	pub empty: E
}

impl<B, S, E> Decorator<B> for ExposedDecorator<B, S, E> where B: Target, S: BlockMatcher<B>, E: BlockMatcher<B> {
	fn generate(&self, quad: &mut QuadMut<B>, _: &mut Random, position: QuadPosition) -> Result {
		if !self.stone.matches(quad.get(position)) {
			return Ok(());
		}

		match position.offset(dir::Down) {
			Some(below) => if !self.stone.matches(quad.get(below)) {
				return Ok(())
			},
			None => return Ok(())
		}

		match position.offset(dir::Up) {
			Some(above) => if !self.stone.matches(quad.get(above)) {
				return Ok(())
			},
			None => return Ok(())
		}

		let mut stone = 0;
		let mut empty = 0;

		if let Some(position) = position.offset(dir::MinusX) {
			let block = quad.get(position);

			if self.stone.matches(block) { stone += 1; }
			if self.empty.matches(block) { empty += 1; }
		} else {
			empty += 1;
		}

		if let Some(position) = position.offset(dir::PlusX) {
			let block = quad.get(position);

			if self.stone.matches(block) { stone += 1; }
			if self.empty.matches(block) { empty += 1; }
		} else {
			empty += 1;
		}

		if let Some(position) = position.offset(dir::MinusZ) {
			let block = quad.get(position);

			if self.stone.matches(block) { stone += 1; }
			if self.empty.matches(block) { empty += 1; }
		} else {
			empty += 1;
		}

		if let Some(position) = position.offset(dir::PlusZ) {
			let block = quad.get(position);

			if self.stone.matches(block) { stone += 1; }
			if self.empty.matches(block) { empty += 1; }
		} else {
			empty += 1;
		}

		if stone == 3 && empty == 1 {
			quad.set_immediate(position, &self.block);
		}

		Ok(())
	}
}