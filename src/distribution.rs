use rng::JavaRng;

/// A random distribution.
pub trait Distribution {
	fn next(&self, rng: &mut JavaRng) -> i32;
}

fn default_chance() -> i32 {
	1
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Chance<D> where D: Distribution {
	/// Chance for this distribution to return its value instead of 0.
	/// Represented as probability = 1 / chance.
	/// A chance of "1" does not call the Chance RNG, and acts as if it passed.
	#[serde(default = "default_chance")]
	pub chance: i32,
	/// If true, then the Chance RNG is called after the payload RNG.
	pub bailout_after: bool,
	pub base: D
}

impl<D> Distribution for Chance<D> where D: Distribution {
	fn next(&self, rng: &mut JavaRng) -> i32 {
		if self.bailout_after {
			let payload = self.base.next(rng);

			if self.chance <= 1 {
				payload
			} else if rng.next_i32(self.chance) == 0 {
				payload
			} else {
				0
			}
		} else {
			if self.chance <= 1 {
				self.base.next(rng)
			} else if rng.next_i32(self.chance) == 0 {
				self.base.next(rng)
			} else {
				0
			}
		}
	}
}

/// Baseline distribution. This should be general enough to fit most use cases.
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum Baseline {
	Constant(i32),
	Linear(Linear),
	Packed2(Packed2),
	Packed3(Packed3),
	Centered(Centered)
}

impl Distribution for Baseline {
	fn next(&self, rng: &mut JavaRng) -> i32 {
		match *self {
			Baseline::Constant(x) => x,
			Baseline::Linear(ref linear) => linear.next(rng),
			Baseline::Packed2(ref packed2) => packed2.next(rng),
			Baseline::Packed3(ref packed3) => packed3.next(rng),
			Baseline::Centered(ref centered) => centered.next(rng)
		}
	}
}

impl Distribution for i32 {
	fn next(&self, _: &mut JavaRng) -> i32 {
		*self
	}
}

/// Plain old linear distribution, with a minimum and maximum.
#[derive(Debug, Serialize, Deserialize)]
pub struct Linear {
	pub min: i32,
	pub max: i32
}

impl Distribution for Linear {
	fn next(&self, rng: &mut JavaRng) -> i32 {
		self.min + rng.next_i32(self.max - self.min + 1)
	}
}

/// Distribution that packs more values to the minimum value. This is based on 2 RNG iterations.
#[derive(Debug, Serialize, Deserialize)]
pub struct Packed2 {
	pub min: i32,
	/// Minimum height passed to the second RNG call (the linear call).
	pub linear_start: i32,
	pub max: i32
}

impl Distribution for Packed2 {
	fn next(&self, rng: &mut JavaRng) -> i32 {
		let initial = rng.next_i32(self.max - self.linear_start + 2);

		self.min + rng.next_i32(initial + self.linear_start - self.min)
	}
}

/// Distribution that packs more values to the minimum value. This is based on 3 RNG iterations, and is more extreme.
/// The average is around `(max+1)/8 - 1`, a simplified form of `(max+1)/2³ - 1`.
#[derive(Debug, Serialize, Deserialize)]
pub struct Packed3 {
	pub max: i32
}

impl Distribution for Packed3 {
	fn next(&self, rng: &mut JavaRng) -> i32 {
		let result = rng.next_i32(self.max + 1);
		let result = rng.next_i32(result + 1);
		rng.next_i32(result + 1)
	}
}

/// Distribution centered around a certain point, with a maximum variance.
#[derive(Debug, Serialize, Deserialize)]
pub struct Centered {
	pub center: i32,
	pub radius: i32
}

impl Distribution for Centered {
	fn next(&self, rng: &mut JavaRng) -> i32 {
		rng.next_i32(self.radius) + rng.next_i32(self.radius) + self.center - self.radius
	}
}