use rng::JavaRng;

const NOTCH_PI: f32 = 3.141593; // TODO: Check
const PI_DIV_2: f32 = 1.570796;
const MIN_H_SIZE: f64 = 1.5;

#[derive(Debug)]
pub struct Caves {
	state: JavaRng, 
	chunk: (i32, i32), 
	from: (i32, i32),
	remaining: i32,
	extra: Option<(i32, (f64, f64, f64))>
}

impl Caves {
	pub fn for_chunk(mut state: JavaRng, chunk: (i32, i32), from: (i32, i32)) -> Caves {
		// Many chained RNG calls allow high values, but make most values low. 
		// Appears as the right half of a normal distribution.
		// Gaah borrow checker
		let mut remaining = state.next_i32(40);
		remaining = state.next_i32(remaining + 1);
		remaining = state.next_i32(remaining + 1);
		
		// Make many chunks not spawn cave starts at all, otherwise the world would look like swiss cheese. 
		// Note that caves starting in other chunks can still carve through this chunk.
		// Offsets the fact that a single cave start can branch many times.
		if state.next_i32(15) != 0 {
			remaining = 0;
		}
		
		Caves { state, chunk, from, remaining, extra: None }
	}
}

impl Iterator for Caves {
	type Item = Start;
	
	fn next(&mut self) -> Option<Start> {
		if self.remaining == 0 {
			return None;
		}
		
		self.remaining -= 1;
		
		if let &mut Some((ref mut extra, orgin)) = &mut self.extra {
			if *extra > 0 {
				*extra -= 1;
				
				return Some(Start::normal(&mut self.state, self.chunk, orgin));
			}
		}
		
		self.extra = None;
		
		let     x = self.state.next_i32(16);
		let mut y = self.state.next_i32(120);
		        y = self.state.next_i32(y + 8);
		let     z = self.state.next_i32(16);
		
		let orgin = (
			(self.from.0 * 16 + x) as f64, 
			y                      as f64, 
			(self.from.1 * 16 + z) as f64
		);
		
		if self.state.next_i32(4) == 0 {
			let circular = Start::circular(&mut self.state, self.chunk, orgin);
			let extra = 1 + self.state.next_i32(4);
			
			self.remaining += extra;
			self.extra = Some((extra, orgin));
			
			Some(circular)
		} else {
			Some(Start::normal(&mut self.state, self.chunk, orgin))
		}
	}
}

#[derive(Debug, Default)]
struct SystemSize {
	current: Option<i32>,
	max:     Option<i32>
}

impl SystemSize {
	fn into_state(&self, max_chunk_radius: i32, rng: &mut JavaRng) -> SystemSizeState {
		let max_block_radius = max_chunk_radius * 16 - 16;
		
		let max = self.max.unwrap_or_else(|| max_block_radius - rng.next_i32(max_block_radius / 4));
		
		let (current, split_threshold) = match self.current {
			Some(current) => (current, Some(rng.next_i32(max / 2) + max / 4)),
			None          => (max / 2, None)
		};
		
		SystemSizeState { max, current, split_threshold }
	}
}

#[derive(Debug)]
struct SystemSizeState {
	current: i32,
	max:     i32,
	/// At this point, the cave will split into 2.
	split_threshold: Option<i32>
}

#[derive(Debug, Copy, Clone)]
struct Position {
	/// Position of the chunk being carved
	chunk: (i32, i32),
	/// Absolute block position in the world
	block: (f64, f64, f64),
	/// Horizontal angle
	yaw: f32,
	/// Vertical angle
	pitch: f32,
	/// Rate of change for the horizontal angle
	yaw_velocity: f32,
	/// Rate of change for the vertical angle
	pitch_velocity: f32
}

#[derive(Debug)]
pub struct Start {
	position: Position,
	max_width: f32,
	size: SystemSize,
	/// 1.0 = normal, 0.5 = smooshed caves (flat rooms)
	vertical_multiplier: f64,
	seed: i64
}

impl Start {
	fn normal(rng: &mut JavaRng, chunk: (i32, i32), block: (f64, f64, f64)) -> Self {
		Start {
			position: Position {
				chunk,
				block,
				yaw: rng.next_f32() * NOTCH_PI * 2.0,
				pitch: (rng.next_f32() - 0.5) / 4.0,
				yaw_velocity: 0.0,
				pitch_velocity: 0.0
			},
			max_width: rng.next_f32() * 2.0 + rng.next_f32(),
			size: SystemSize {
				current: Some(0),
				max: None
			},
			vertical_multiplier: 1.0,
			seed: rng.next_i64()
		}
	}
	
	fn circular(rng: &mut JavaRng, chunk: (i32, i32), block: (f64, f64, f64)) -> Self {
		Start {
			position: Position {
				chunk,
				block,
				yaw: 0.0,
				pitch: 0.0,
				yaw_velocity: 0.0,
				pitch_velocity: 0.0
			},
			max_width: 1.0 + rng.next_f32() * 6.0,
			size: SystemSize::default(),
			vertical_multiplier: 0.5,
			seed: rng.next_i64()
		}
	}
	
	fn into_tunnel(&self) -> Tunnel {
		Tunnel {
			state: JavaRng::new(self.seed),
			position: self.position,
			size_state: unimplemented!(),
			yaw_keep: unimplemented!(),
			max_width: self.max_width,
			vertical_multiplier: self.vertical_multiplier
		}
	}
}

struct Tunnel {
	state: JavaRng,
	position: Position,
	size_state: SystemSizeState,
	/// Decided by is_steep_cave
	yaw_keep: f32,
	max_width: f32,
	vertical_multiplier: f64
}