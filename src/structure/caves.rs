use rng::JavaRng;
use trig::TrigLookup;
use std::cmp::{min, max};
use distribution::rarity::{Rarity, HalfNormal3, Rare};

const NOTCH_PI: f32 = 3.141593; // TODO: Check
const PI_DIV_2: f32 = 1.570796;
const MIN_H_SIZE: f64 = 1.5;

/// Make many chunks not spawn cave starts at all, otherwise the world would look like swiss cheese. 
/// Note that caves starting in other chunks can still carve through this chunk.
/// Offsets the fact that a single cave start can branch many times.
/// Also make most chunks that do contain caves contain few, but have the potential to contain many.
static RARITY: Rare<HalfNormal3> = Rare {
	base: HalfNormal3 { max: 39 },
	rarity: 15
};

fn floor_capped(t: f64) -> i32 {
	t.floor().max(-2147483648.0).min(2147483647.0) as i32
}

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
		let remaining = RARITY.get(&mut state);
		
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
	fn to_state(&self, rng: &mut JavaRng, max_chunk_radius: i32, blob_size_factor: f32) -> SystemSizeState {
		let max_block_radius = max_chunk_radius * 16 - 16;
		
		let max = self.max.unwrap_or_else(|| max_block_radius - rng.next_i32(max_block_radius / 4));
		let split = rng.next_i32(max / 2) + max / 4;
		let split = if blob_size_factor > 1.0 {Some(split)} else {None};
		
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

impl SystemSizeState {
	fn to_size(&self) -> SystemSize {
		SystemSize {
			current: Some(self.current),
			max: Some(self.max)
		}
	}
}

#[derive(Debug, Copy, Clone)]
struct Position {
	/// Position of the chunk being carved
	chunk: (i32, i32),
	/// Block position of the center of the generated chunk.
	chunk_center: (f64, f64),
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
	
	fn distance_from_chunk_squared(&self) -> f64 {
		let distance_x = self.block.0 - self.chunk_center.0;
		let distance_z = self.block.2 - self.chunk_center.1;
		
		distance_x * distance_x + distance_z * distance_z
	}
	
	fn out_of_chunk(&self, blob: &BlobSize) -> bool {
		let horizontal_diameter = blob.horizontal_diameter();
		
		self.block.0 < self.chunk_center.0 - 16.0 - horizontal_diameter ||
		self.block.2 < self.chunk_center.1 - 16.0 - horizontal_diameter ||
		self.block.0 > self.chunk_center.0 + 16.0 + horizontal_diameter ||
		self.block.2 > self.chunk_center.1 + 16.0 + horizontal_diameter
	}
	
	fn blob(&self, size: BlobSize) -> Blob {
		let lower = (
			floor_capped(self.block.0 - size.horizontal) - self.chunk.0 * 16 - 1,
			floor_capped(self.block.1 - size.vertical)                       - 1,
			floor_capped(self.block.2 - size.horizontal) - self.chunk.1 * 16 - 1
		);
		
		let upper = (
			floor_capped(self.block.0 + size.horizontal) - self.chunk.0 * 16 + 1,
			floor_capped(self.block.1 + size.vertical)                       + 1,
			floor_capped(self.block.2 + size.horizontal) - self.chunk.1 * 16 + 1
		);
		
		Blob {
			center: self.block,
			size,
			lower: (
				max(lower.0, 0),
				max(lower.1, 1),
				max(lower.2, 0)
			),
			upper: (
				min(upper.0, 16),
				min(upper.1, 120),
				min(upper.2, 16)
			)
		}
	}
}

#[derive(Debug)]
pub struct Start {
	position: Position,
	blob_size_factor: f32,
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
				chunk_center: ((chunk.0 * 16 + 8) as f64, (chunk.1 * 16 + 8) as f64),
				block,
				yaw: rng.next_f32() * NOTCH_PI * 2.0,
				pitch: (rng.next_f32() - 0.5) / 4.0,
				yaw_velocity: 0.0,
				pitch_velocity: 0.0
			},
			blob_size_factor: rng.next_f32() * 2.0 + rng.next_f32(),
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
				chunk_center: ((chunk.0 * 16 + 8) as f64, (chunk.1 * 16 + 8) as f64),
				block,
				yaw: 0.0,
				pitch: 0.0,
				yaw_velocity: 0.0,
				pitch_velocity: 0.0
			},
			blob_size_factor: 1.0 + rng.next_f32() * 6.0,
			size: SystemSize::default(),
			vertical_multiplier: 0.5,
			seed: rng.next_i64()
		}
	}
	
	pub fn to_tunnel(&self, max_chunk_radius: i32) -> Tunnel {
		let mut state = JavaRng::new(self.seed);
		let size_state = self.size.to_state(&mut state, max_chunk_radius, self.blob_size_factor);
		let pitch_keep = if state.next_i32(6) == 0 { 0.92 } else { 0.7 };
		
		Tunnel { state, position: self.position, size_state, pitch_keep, blob_size_factor: self.blob_size_factor, vertical_multiplier: self.vertical_multiplier }
	}
}

enum Outcome {
	Split,
	Constrict,
	Unreachable,
	OutOfBounds,
	Carve(BlobSize)
}

#[derive(Debug)]
pub struct Tunnel {
	state: JavaRng,
	position: Position,
	size_state: SystemSizeState,
	/// 0.92 = Steep, 0.7 = Normal
	pitch_keep: f32,
	blob_size_factor: f32,
	vertical_multiplier: f64
}

impl Tunnel {
	fn split(&mut self, caves: &mut Caves) -> (Start, Start) {
		// https://bugs.mojang.com/browse/MC-7196
		// Second bug resulting in chunk cliffs, that we have to recreate
		// The tunnel splitting calls back to the root RNG, causing discontinuities if is_chunk_unreachable() returns true before the cave splits.
		// If the is_chunk_unreachable optimization is disabled, this issue doesn't occur.
		// It also wrecks the nice, clean iterator implementation, as we have to recreate the bug. Ugh.
		// Luckily, we can still use Rust lifetimes to prevent someone from accidentally forgetting this fact.
		
		(Start {
			position: Position {
				chunk: self.position.chunk,
				chunk_center: self.position.chunk_center,
				block: self.position.block,
				yaw: self.position.yaw - PI_DIV_2,
				pitch: self.position.pitch / 3.0,
				yaw_velocity: 0.0,
				pitch_velocity: 0.0
			},
			blob_size_factor: self.state.next_f32() * 0.5 + 0.5,
			size: self.size_state.to_size(),
			vertical_multiplier: 1.0,
			seed: caves.state.next_i64() 
		}, Start {
			position: Position {
				chunk: self.position.chunk,
				chunk_center: self.position.chunk_center,
				block: self.position.block,
				yaw: self.position.yaw + PI_DIV_2,
				pitch: self.position.pitch / 3.0,
				yaw_velocity: 0.0,
				pitch_velocity: 0.0
			},
			blob_size_factor: self.state.next_f32() * 0.5 + 0.5,
			size: self.size_state.to_size(),
			vertical_multiplier: 1.0,
			seed: caves.state.next_i64()
		})
	}
	
	fn is_chunk_unreachable(&self) -> bool {
		// https://bugs.mojang.com/browse/MC-7200
		// First bug resulting in chunk cliffs, that we should recreate.
		// Using addition/subtraction with distance squared math is invalid.
		
		let remaining = (self.size_state.max - self.size_state.current) as f64;
		
		// Conservative buffer distance that accounts for the size of each carved part.
		let buffer = (self.blob_size_factor * 2.0 + 16.0) as f64; 
		
		// Invalid: Subtraction from distance squared.
		self.position.distance_from_chunk_squared() - remaining * remaining > buffer * buffer
	}
	
	fn get_blob_size(&self, trig: &TrigLookup) -> BlobSize {
		BlobSize::from_horizontal(
			MIN_H_SIZE + (trig.sin(self.size_state.current as f32 * NOTCH_PI / self.size_state.max as f32) * self.blob_size_factor) as f64, 
			self.vertical_multiplier
		)
	}
}

#[derive(Debug, Copy, Clone)]
struct BlobSize {
	/// Radius on the X/Z axis
	horizontal: f64,
	/// Radius on the Y axis
	vertical: f64
}

impl BlobSize {
	fn from_horizontal(horizontal: f64, vertical_multiplier: f64) -> Self {
		BlobSize {
			horizontal,
			vertical: horizontal * vertical_multiplier
		}
	}
	
	fn horizontal_diameter(&self) -> f64 {
		self.horizontal * 2.0
	}
}

struct Blob {
	/// Center of the blob
	center: (f64, f64, f64),
	/// Size of the blob
	size: BlobSize,
	/// Lower bounds of the feasible region, in chunk coordinates: [0,16), [0,128), [0,16)
	lower: (i32, i32, i32),
	/// Upper bounds of the feasible region, in chunk coordiantes: [0,16), [0,128), [0,16)
	upper: (i32, i32, i32)
}