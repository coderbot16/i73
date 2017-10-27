use rng::JavaRng;
use chunk::grouping::{Moore, Result};
use chunk::storage::Target;
use distribution::height::HeightDistribution;
use distribution::rarity::Rarity;
use std::marker::PhantomData; 

pub mod dungeon;
pub mod vein;
pub mod large_tree;
pub mod lake;
pub mod tree;

// TODO: MultiDispatcher

pub struct Dispatcher<H, D, R, B> where H: HeightDistribution, D: Decorator<B>, R: Rarity, B: Target {
	pub height_distribution: H,
	pub rarity: R,
	pub decorator: D,
	pub phantom: PhantomData<B>
}

impl<H, D, R, B> Dispatcher<H, D, R, B> where H: HeightDistribution, D: Decorator<B>, R: Rarity, B: Target {
	pub fn generate(&self, moore: &mut Moore<B>, rng: &mut JavaRng) -> Result<bool> {
		for _ in 0..self.rarity.get(rng) {
			let at = (rng.next_i32(16), self.height_distribution.get(rng), rng.next_i32(16));
			
			self.decorator.generate(moore, rng, at)?;
		}
		
		Ok(true)
	}
}

pub trait Decorator<B> where B: Target {
	fn generate(&self, moore: &mut Moore<B>, rng: &mut JavaRng, position: (i32, i32, i32)) -> Result<bool>;
}