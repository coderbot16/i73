use rng::JavaRng;
use trig::TrigLookup;

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
	fn to_state(&self, rng: &mut JavaRng, max_chunk_radius: i32, max_width: f32) -> SystemSizeState {
		let max_block_radius = max_chunk_radius * 16 - 16;
		
		let max = self.max.unwrap_or_else(|| max_block_radius - rng.next_i32(max_block_radius / 4));
		let split = rng.next_i32(max / 2) + max / 4;
		let split = if max_width > 1.0 {Some(split)} else {None};
		
		let (current, split_threshold, max_iter, can_constrict) = match self.current {
			Some(current) => (current, split, max, true ),
			None          => (max / 2, None,  1,   false)
		};
		
		SystemSizeState { max, current, split_threshold, max_iter, can_constrict }
	}
}

#[derive(Debug)]
struct SystemSizeState {
	current:    i32,
	max:        i32,
	max_iter:   i32,
	/// At this point, the tunnel will split into 2. None if it won't split.
	split_threshold: Option<i32>,
	/// Whether the tunnel can randomly not carve (25% chance by default), which varies the radius of the cave. 
	/// It can also result in a very thin hole between 2 parts of the same cave.
	can_constrict: bool
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

impl Position {
	fn step(&mut self, rng: &mut JavaRng, trig: &TrigLookup, pitch_keep: f32) {
		let cos_pitch = trig.cos(self.pitch);
		
		self.block.0 += (trig.cos(self.yaw) * cos_pitch) as f64;
		self.block.1 +=  trig.sin(self.pitch)            as f64;
		self.block.2 += (trig.sin(self.yaw) * cos_pitch) as f64;
		
		self.pitch *= pitch_keep;
		self.pitch += self.pitch_velocity * 0.1;
		self.yaw += self.yaw_velocity * 0.1;
		
		self.pitch_velocity *= 0.9;
		self.yaw_velocity *= 0.75;
		self.pitch_velocity += (rng.next_f32() - rng.next_f32()) * rng.next_f32() * 2.0;
		self.yaw_velocity += (rng.next_f32() - rng.next_f32()) * rng.next_f32() * 4.0;
	}
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
	
	pub fn to_tunnel(&self, max_chunk_radius: i32) -> Tunnel {
		let mut state = JavaRng::new(self.seed);
		let size_state = self.size.to_state(&mut state, max_chunk_radius, self.max_width);
		let pitch_keep = if state.next_i32(6) == 0 { 0.92 } else { 0.7 };
		
		Tunnel { state, position: self.position, size_state, pitch_keep, max_width: self.max_width, vertical_multiplier: self.vertical_multiplier }
	}
}

#[derive(Debug)]
pub struct Tunnel {
	state: JavaRng,
	position: Position,
	size_state: SystemSizeState,
	/// 0.92 = Steep, 0.7 = Normal
	pitch_keep: f32,
	max_width: f32,
	vertical_multiplier: f64
}