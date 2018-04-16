use rng::JavaRng;
use vocs::view::QuadMut;
use vocs::position::{ColumnPosition, QuadPosition};
use vocs::indexed::Target;
use distribution::Distribution;

pub mod dungeon;
pub mod vein;
pub mod clump;
// pub mod large_tree;
pub mod lake;
// pub mod tree;

// TODO: MultiDispatcher

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct Spilled(pub QuadPosition);
pub type Result = ::std::result::Result<(), Spilled>;

pub struct Dispatcher<H, D, R, B> where H: Distribution, D: Decorator<B>, R: Distribution, B: Target {
	pub height_distribution: H,
	pub rarity: R,
	pub decorator: D,
	pub phantom: ::std::marker::PhantomData<B>
}

impl<H, D, R, B> Dispatcher<H, D, R, B> where H: Distribution, D: Decorator<B>, R: Distribution, B: Target {
	pub fn generate(&self, quad: &mut QuadMut<B>, rng: &mut JavaRng) -> Result {
		for _ in 0..self.rarity.next(rng) {
			let at = ColumnPosition::new(
				rng.next_i32(16) as u8,
				self.height_distribution.next(rng) as u8,
				rng.next_i32(16) as u8
			);
			
			self.decorator.generate(quad, rng, QuadPosition::from_centered(at))?;
		}
		
		Ok(())
	}
}

pub trait Decorator<B> where B: Target {
	fn generate(&self, quad: &mut QuadMut<B>, rng: &mut JavaRng, position: QuadPosition) -> Result;
}