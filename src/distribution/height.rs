use rng::JavaRng;

const DEFAULT: Linear = Linear { min: 0, max: 127 };

trait HeightDistribution {
	fn height(&self, rng: &mut JavaRng) -> i32;
}

/// Distribution centered around a certain point.
struct Centered {
	center: i32,
	radius: i32
}

impl HeightDistribution for Centered {
	fn height(&self, rng: &mut JavaRng) -> i32 {
		rng.next_i32(self.radius) + rng.next_i32(self.radius) + self.center - self.radius
	}
}

/// Distribution that allows high height but has a low average height.
struct DepthPacked {
	min: i32,
	// TODO: Investigate this.
	start: i32,
	max: i32
}

impl HeightDistribution for DepthPacked {
	fn height(&self, rng: &mut JavaRng) -> i32 {
		let initial = rng.next_i32(self.max - self.start + 2);
		
		self.min + rng.next_i32(initial + self.start - self.min)
	}
}

/// Plain old linear height distribution, with a minimum and maximum.
struct Linear {
	min: i32,
	max: i32
}

impl HeightDistribution for Linear {
	fn height(&self, rng: &mut JavaRng) -> i32 {
		self.min + rng.next_i32(self.max - self.min + 1)
	}
}