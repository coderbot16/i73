pub struct TrigLookup {
	// There's no way 262KB of data will fit on smaller stacks.
	sin: Box<[f32]>
}

impl TrigLookup {
	pub fn new() -> Self {
		let mut data = Vec::with_capacity(65536);
		for i in 0..65536 {
			data.push(((i as f64) * 3.141592653589793 * 2.0 / 65536.0).sin() as f32);
		}
		
		TrigLookup { sin: data.into_boxed_slice() }
	}
	
	pub fn sin(&self, f: f32) -> f32 {
		self.sin[(((f * 10430.38) as i32) & 0xFFFF) as usize]
	}
	
	pub fn cos(&self, f: f32) -> f32 {
		self.sin[(((f * 10430.38 + 16384.0) as i32) & 0xFFFF) as usize]
	}
}