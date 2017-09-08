const F32_DIV: f32 = (1u32 << 24) as f32;
const F64_DIV: f64 = (1u64 << 53) as f64;

#[derive(Debug)]
pub struct JavaRng {
	pub seed: i64
}

impl JavaRng {
	pub fn new(seed: i64) -> Self {
		JavaRng {seed: (seed ^ 0x5DEECE66D) & ((1 << 48) - 1)}
	}
	
	fn next(&mut self, bits: u8) -> i32 {
		if bits > 48 {
			panic!("Too many bits!")
		}
		
		self.seed = (self.seed.wrapping_mul(0x5DEECE66D).wrapping_add(0xB)) & ((1 << 48) - 1);
		(self.seed >> (48 - bits)) as i32
	}
	
	pub fn next_i32(&mut self, max: i32) -> i32 {
		if max <= 0 {
			panic!("Maximum must be > 0")
		}
		
		if (max & -max) == max  {// i.e., n is a power of 2
			let max = max as u64;
			
			//println!("rng: next_i32({}) => {}", max, ((max * (self.next(31) as u64)) >> 31) as i32);
			
			return ((max * (self.next(31) as u64)) >> 31) as i32;
		}
     
		let mut bits = self.next(31) as i32;
		let mut val = bits % max;
		
		while bits - val + (max - 1) < 0 {
			bits = self.next(31) as i32;
			val = bits % max;
		}
		
		//println!("rng: next_i32({}) => {}", max, val);
		
		val
	}
	
	pub fn next_i64(&mut self) -> i64 {
		((self.next(32) as i64) << 32) | (self.next(32) as i64)
	}
	
	pub fn next_f32(&mut self) -> f32 {
		(self.next(24) as f32) / F32_DIV
	}
	
	pub fn next_f64(&mut self) -> f64 {
		let high = (self.next(26) as i64) << 27;
		let low = self.next(27) as i64;
		
		((high | low) as f64) / F64_DIV
	}
}