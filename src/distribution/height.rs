use rng::JavaRng;

pub const DEFAULT: Linear = Linear { min: 0, max: 127 };

pub trait HeightDistribution {
	fn get(&self, rng: &mut JavaRng) -> i32;
}

/// Distribution centered around a certain point.
pub struct Centered {
	pub center: i32,
	pub radius: i32
}

impl HeightDistribution for Centered {
	fn get(&self, rng: &mut JavaRng) -> i32 {
		rng.next_i32(self.radius) + rng.next_i32(self.radius) + self.center - self.radius
	}
}

/// Distribution that allows high height but has a low average height.
pub struct DepthPacked {
	pub min: i32,
	// TODO: Investigate this.
	pub linear_start: i32,
	pub max: i32
}

impl HeightDistribution for DepthPacked {
	fn get(&self, rng: &mut JavaRng) -> i32 {
		let initial = rng.next_i32(self.max - self.linear_start + 2);
		
		self.min + rng.next_i32(initial + self.linear_start - self.min)
	}
}

/// Plain old linear height distribution, with a minimum and maximum.
pub struct Linear {
	pub min: i32,
	pub max: i32
}

impl HeightDistribution for Linear {
	fn get(&self, rng: &mut JavaRng) -> i32 {
		self.min + rng.next_i32(self.max - self.min + 1)
	}
}