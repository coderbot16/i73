const F32_DIV: f32 = (1u32 << 24) as f32;
const F64_DIV: f64 = (1u64 << 53) as f64;

/// Implementation of a random number generator matching the implementation in Java. Used very commonly in all versions of the Minecraft worldgen.
#[derive(Debug)]
pub struct JavaRng {
	pub seed: i64
}

impl JavaRng {
	/// Initializes the RNG with a seed. This is NOT the same as creating it raw, as the seed undergoes some transformation first.
	pub fn new(seed: i64) -> Self {
		JavaRng {seed: (seed ^ 0x5DEECE66D) & ((1 << 48) - 1)}
	}
	
	/// Steps the RNG by one, returning up to 48 bits.
	fn next(&mut self, bits: u8) -> i32 {
		if bits > 48 {
			panic!("Too many bits!")
		}
		
		self.seed = (self.seed.wrapping_mul(0x5DEECE66D).wrapping_add(0xB)) & ((1 << 48) - 1);
		(self.seed >> (48 - bits)) as i32
	}
	
	/// Returns an i32 in the range [0, max). 
	pub fn next_i32(&mut self, max: i32) -> i32 {
		if max <= 0 {
			panic!("Maximum must be > 0")
		}
		
		if (max & -max) == max  {// i.e., n is a power of 2
			let max = max as u64;
			
			return ((max.wrapping_mul(self.next(31) as u64)) >> 31) as i32;
		}
     
		let mut bits = self.next(31) as i32;
		let mut val = bits % max;
		
		while bits - val + (max - 1) < 0 {
			bits = self.next(31) as i32;
			val = bits % max;
		}
		
		val
	}
	
	/// Returns an i64. There are only 2^48 possible results from this function, as JavaRng has a 48-bit state.
	pub fn next_i64(&mut self) -> i64 {
		((self.next(32) as i64) << 32).wrapping_add(self.next(32) as i64)
	}
	
	/// Returns a f32 uniformly distributed between 0.0 and 1.0.
	pub fn next_f32(&mut self) -> f32 {
		(self.next(24) as f32) / F32_DIV
	}
	
	/// Returns a f64 uniformly distributed between 0.0 and 1.0.
	pub fn next_f64(&mut self) -> f64 {
		let high = (self.next(26) as i64) << 27;
		let low = self.next(27) as i64;
		
		(high.wrapping_add(low) as f64) / F64_DIV
	}
}

/// Knuth's optimal 64-bit multiplier (a) for a LCG, used in MMIX and Newlib.
const KNUTH_A: i64 = 6364136223846793005;

/// Knuth's optimal 64-bit increment (a) for a LCG, used in MMIX.
const KNUTH_C: i64 = 1442695040888963407;

/// Steps a LCG using Knuth's optimal A and C constants, and with a modulus of 2^64.
fn step_knuth(state: i64) -> i64 {
	state.wrapping_mul(KNUTH_A).wrapping_add(KNUTH_C)
}

/// Steps a LCG with an A value determined by the output of the step_knuth function and a C value provided by the caller. Modulus is still 2^64.
fn step_salted(state: i64, c: i64) -> i64 {
	state.wrapping_mul(step_knuth(state)).wrapping_add(c)
}

/// Notch's custom RNG, that can be initialized from a position.
/// Used commonly in Beta 1.8 and later worldgen for biome generation, among other things.
#[derive(Debug)]
pub struct NotchRng {
	/// Initial value assigned to the RNG at a position before mixing in the coordinates.
	pub initial: i64,
	/// The current internal state of the RNG, initialized using `NotchRng::init_at` and modified with the next... functions.
	pub state:   i64
}

impl NotchRng {
	/// Initialize a NotchRng. The seed value usually represents the world seed. 
	/// The salt is a unique value that differentiates this RNG from other instances with the same world seed.
	pub fn new(salt: i64, seed: i64) -> Self {
		let mut primary = salt;
		
		primary = step_salted(primary, salt);
		primary = step_salted(primary, salt);
		primary = step_salted(primary, salt);
		
		let mut initial = seed;
		
		initial = step_salted(initial, primary);
		initial = step_salted(initial, primary);
		initial = step_salted(initial, primary);
		
		NotchRng {
			initial,
			state: 0
		}
	}
	
	pub fn init_at(&mut self, x: i64, z: i64) {
		self.state = self.initial;
		
		self.state = step_salted(self.state, x);
		self.state = step_salted(self.state, z);
		self.state = step_salted(self.state, x);
		self.state = step_salted(self.state, z);
	}
	
	/// Steps the RNG forward by one. Unlike JavaRng, this function always returns 40 bits.
	pub fn next(&mut self) -> i64 {
		let result = self.state >> 24;
		
		self.state = step_salted(self.state, self.initial);
		
		result
	}
	
	/// Returns an i32 in the range [0, max). 
	/// Make sure to call `init_at(x, z)` first if calling this from world generation code!
	pub fn next_i32(&mut self, max: i32) -> i32 {
		if max <= 0 {
			panic!("Maximum must be > 0")
		}
		
		// Get a value in the range (-max, max)
		let result = (self.next().wrapping_rem(max as i64)) as i32; // TODO: Casting order
		
		// Shift the result into the range [0, max)
		result + if result < 0 { max } else { 0 }
	}
}