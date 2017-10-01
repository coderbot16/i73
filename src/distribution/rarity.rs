use rng::JavaRng;

pub trait Rarity {
	fn get(&self, rng: &mut JavaRng) -> i32;
}


impl Rarity for i32 {
	fn get(&self, _: &mut JavaRng) -> i32 {
		*self
	}
}


/// Half of a Normal distribution with 3 iterations of RNG. Average is `(max+1)/8 - 1`, a simplified form of `(max+1)/2³ - 1`.
pub struct HalfNormal3 {
	pub max: i32
}

impl Rarity for HalfNormal3 {
	fn get(&self, rng: &mut JavaRng) -> i32 {
		let result = rng.next_i32(self.max + 1);
		let result = rng.next_i32(result + 1);
		rng.next_i32(result + 1)
	}
}

/// Extends a rarity by normally returning zero, with a chance to return a non zero value.
/// The chance to return a nonzero value is expressed as `1/rarity`.
pub struct Rare<R> where R: Rarity {
	pub base: R,
	pub rarity: i32
}

impl<R> Rarity for Rare<R> where R: Rarity {
	fn get(&self, rng: &mut JavaRng) -> i32 {
		let candidate = self.base.get(rng);
		
		if rng.next_i32(self.rarity) != 0 {candidate} else {0}
	}
}

// TODO: Flowers, Grass, DeadBush, Cactus