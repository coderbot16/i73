use rng::JavaRng;
use trig::TrigLookup;
use std::cmp::{min, max};
use distribution::rarity::{Rarity, HalfNormal3, Rare};
use structure::StructureGenerator;
use chunk::storage::Target;
use chunk::grouping::{Column, ColumnBlocks, ColumnPalettes, ColumnAssociation};
use chunk::position::BlockPosition;
use chunk::matcher::BlockMatcher;

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

static RARITY_NETHER: Rare<HalfNormal3> = Rare {
	base: HalfNormal3 { max: 9 },
	rarity: 5
};

/// Mimics Java rounding rules and avoids UB from float casts.
fn floor_capped(t: f64) -> i32 {
	t.floor().max(-2147483648.0).min(2147483647.0) as i32
}

struct CavesAssociations<'a, B> where B: 'a + Target {
	carve: ColumnAssociation<'a, B>
}

pub struct CavesGenerator<B, O, C> where B: Target, O: BlockMatcher<B>, C: BlockMatcher<B> {
	pub lookup: TrigLookup,
	pub carve:  B,
	pub ocean:  O,
	pub carvable: C
}

impl<B, O, C> CavesGenerator<B, O, C> where B: Target, O: BlockMatcher<B>, C: BlockMatcher<B> {
	fn carve_blob(&self, blob: Blob, associations: &CavesAssociations<B>, blocks: &mut ColumnBlocks, palette: &ColumnPalettes<B>, chunk: (i32, i32)) {
		let chunk_block = ((chunk.0 * 16) as f64, (chunk.1 * 16) as f64);
		
		// Try to make sure that we don't carve into the ocean.
		// However, this misses chunk boundaries - there is no easy way to fix this.
		
		for z in blob.lower.2..blob.upper.2 {
			for x in blob.lower.0..blob.upper.0 {
				let mut y = (blob.upper.1 + 1) as i32;
				
				while y >= (blob.lower.1 - 1) as i32 {
					if y < 0 || y >= 128 {
						y -= 1;
						continue;
					}
					
					let block = BlockPosition::new(x, y as u8, z);
					
					if let Ok(candidate) = blocks.get(block, palette).target() {
						if self.ocean.matches(candidate) {
							return;
						}
					}
					
					// Optimization: Only check the edges.
					if    y != (blob.lower.1 - 1) as i32 
					   && x != blob.lower.0 && x != blob.upper.0 - 1 
					   && z != blob.lower.2 && z != blob.upper.2 - 1 {
					   	// If it ain't on any of the other 5 sides, check the bottom and skip the interior of the volume.
						y = blob.lower.1 as i32;
					}
					
					y -= 1;
				}
			}
		}
		
		for z in blob.lower.2..blob.upper.2 {
			for x in blob.lower.0..blob.upper.0 {
				for y in blob.lower.1..blob.upper.1 {
					let position = BlockPosition::new(x, y, z);
					
					let block = (x as f64, y as f64, z as f64);
				
					let scaled = (
						(block.0 + chunk_block.0 + 0.5 - blob.center.0) / blob.size.horizontal,
						(block.1                 + 0.5 - blob.center.1) / blob.size.vertical,
						(block.2 + chunk_block.1 + 0.5 - blob.center.2) / blob.size.horizontal
					);
					
					// TODO: Pull down grass and other blocks.
					
					// Test if the block is within the blob region. Additionally, the y > -0.7 check makes the floors flat.
					if scaled.1 > -0.7 && scaled.0 * scaled.0 + scaled.1 * scaled.1 + scaled.2 * scaled.2 < 1.0 {
						if let Ok(candidate) = blocks.get(position, palette).target() {
							if !self.carvable.matches(candidate) {
								continue;
							}
						}
						
						blocks.set(position, &associations.carve);
					}
				}
			}
		}
	}
	
	fn carve_tunnel(&self, mut tunnel: Tunnel, caves: &mut Caves, associations: &CavesAssociations<B>, blocks: &mut ColumnBlocks, palette: &ColumnPalettes<B>, chunk: (i32, i32), from: (i32, i32), radius: i32) {
		loop {
			let outcome = tunnel.step(&self.lookup);
			
			match outcome {
				Outcome::Split       => {
					let (a, b) = tunnel.split(caves);
					
					self.carve_tunnel(a, caves, associations, blocks, palette, chunk, from, radius);
					self.carve_tunnel(b, caves, associations, blocks, palette, chunk, from, radius);
					
					return
				},
				Outcome::Constrict   => (),
				Outcome::Unreachable => return,
				Outcome::OutOfChunk  => (),
				Outcome::Carve(blob) => self.carve_blob(blob, associations, blocks, palette, chunk),
				Outcome::Done        => return
			}
		}
	}
}

impl<B, O, C> StructureGenerator<B> for CavesGenerator<B, O, C> where B: Target, O: BlockMatcher<B>, C: BlockMatcher<B> {
	fn generate(&self, random: JavaRng, column: &mut Column<B>, chunk: (i32, i32), from: (i32, i32), radius: i32) {
		let mut caves = Caves::for_chunk(random, chunk, from, radius, &self.lookup);
		
		column.ensure_available(self.carve.clone());
		
		let (mut blocks, palette) = column.freeze_palettes();
		
		let associations = CavesAssociations {
			carve: palette.reverse_lookup(&self.carve).unwrap()
		};
		
		while let Some(start) = caves.next() {
			match start {
				Start::Tunnel(tunnel) => self.carve_tunnel(tunnel, &mut caves, &associations, &mut blocks, &palette, chunk, from, radius),
				Start::Circular(blob) => if let Some(blob) = blob {
					self.carve_blob(blob, &associations, &mut blocks, &palette, chunk)
				}
			};
		}
	}
}

// TODO: #[derive(Debug)]
pub struct Caves<'a> {
	state: JavaRng, 
	chunk: (i32, i32), 
	from: (i32, i32),
	remaining: i32,
	max_chunk_radius: i32,
	trig: &'a TrigLookup,
	extra: Option<(i32, (f64, f64, f64))>
}

impl<'a> Caves<'a> {
	pub fn for_chunk(mut state: JavaRng, chunk: (i32, i32), from: (i32, i32), radius: i32, trig: &TrigLookup) -> Caves {
		let remaining = RARITY.get(&mut state);
		
		Caves { state, chunk, from, remaining, extra: None, max_chunk_radius: radius, trig }
	}
}

impl<'a> Iterator for Caves<'a> {
	type Item = Start;
	
	fn next(&mut self) -> Option<Start> {
		if self.remaining == 0 {
			return None;
		}
		
		self.remaining -= 1;
		
		if let &mut Some((ref mut extra, orgin)) = &mut self.extra {
			if *extra > 0 {
				*extra -= 1;
				
				return Some(Start::normal(&mut self.state, self.chunk, orgin, self.max_chunk_radius));
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
			let circular = Start::circular(&mut self.state, self.chunk, orgin, self.max_chunk_radius, self.trig);
			let extra = 1 + self.state.next_i32(4);
			
			self.remaining += extra;
			self.extra = Some((extra, orgin));
			
			Some(circular)
		} else {
			Some(Start::normal(&mut self.state, self.chunk, orgin, self.max_chunk_radius))
		}
	}
}

#[derive(Debug)]
pub enum Start {
	Circular(Option<Blob>),
	Tunnel(Tunnel)
}

impl Start {
	fn normal(rng: &mut JavaRng, chunk: (i32, i32), block: (f64, f64, f64), max_chunk_radius: i32) -> Self {
		Start::Tunnel(Tunnel::normal(rng, chunk, block, max_chunk_radius))
	}
	
	fn circular(rng: &mut JavaRng, chunk: (i32, i32), block: (f64, f64, f64), max_chunk_radius: i32, trig: &TrigLookup) -> Self {
		let blob_size_factor = 1.0 + rng.next_f32() * 6.0;
		let mut state = JavaRng::new(rng.next_i64());
		
		let mut size = SystemSize::new(&mut state, 0, max_chunk_radius);
		size.current = size.max / 2;
		
		let size = BlobSize::from_horizontal(
			MIN_H_SIZE + (trig.sin(size.current as f32 * NOTCH_PI / size.max as f32) * blob_size_factor) as f64, 
			0.5
		);
		
		let position = Position::new(chunk, (block.0 + 1.0, block.1, block.2));
		
		Start::Circular(if position.out_of_chunk(&size) {
			None
		} else {
			Some(position.blob(size))
		})
	}
}

#[derive(Debug)]
pub struct Tunnel {
	state: JavaRng,
	position: Position,
	size: SystemSize,
	split: Option<i32>,
	/// 0.92 = Steep, 0.7 = Normal
	pitch_keep: f32,
	blob_size_factor: f32
}

impl Tunnel {
	fn normal(rng: &mut JavaRng, chunk: (i32, i32), block: (f64, f64, f64), max_chunk_radius: i32) -> Self {
		let position = Position::with_angles(chunk, block, rng.next_f32() * NOTCH_PI * 2.0, (rng.next_f32() - 0.5) / 4.0);
		let blob_size_factor = rng.next_f32() * 2.0 + rng.next_f32();
		
		let mut state = JavaRng::new(rng.next_i64());
		
		let size = SystemSize::new(&mut state, 0, max_chunk_radius);
		
		Tunnel { 
			position, 
			size,
			split:      size.split(&mut state, blob_size_factor), 
			pitch_keep: if state.next_i32(6) == 0 { 0.92 } else { 0.7 }, 
			blob_size_factor,
			state
		}
	}
	
	fn split_off(&mut self, rng: &mut JavaRng, yaw_offset: f32) -> Tunnel {
		let position = Position::with_angles(self.position.chunk, self.position.block, self.position.yaw + yaw_offset, self.position.pitch / 3.0);
		let blob_size_factor = self.state.next_f32() * 0.5 + 0.5;
		
		let mut state = JavaRng::new(rng.next_i64());
		
		let size = self.size;
		
		Tunnel { 
			position, 
			size,
			split:      size.split(&mut state, blob_size_factor), 
			pitch_keep: if state.next_i32(6) == 0 { 0.92 } else { 0.7 }, 
			blob_size_factor,
			state
		}
	}
	
	fn split(&mut self, caves: &mut Caves) -> (Tunnel, Tunnel) {
		// https://bugs.mojang.com/browse/MC-7196
		// First bug resulting in chunk cliffs, that we have to recreate
		// The tunnel splitting calls back to the root RNG, causing discontinuities if is_chunk_unreachable() returns true before the cave splits.
		// If the is_chunk_unreachable optimization is disabled, this issue doesn't occur.
		// It also wrecks the nice, clean iterator implementation, as we have to pass the RNG down. Ugh.
		
		(self.split_off(&mut caves.state, -PI_DIV_2), self.split_off(&mut caves.state, PI_DIV_2))
	}
	
	fn is_chunk_unreachable(&self) -> bool {
		// https://bugs.mojang.com/browse/MC-7200
		// Second bug resulting in chunk cliffs, that we have to recreate.
		// Using addition/subtraction with distance squared math is invalid.
		
		let remaining = (self.size.max - self.size.current) as f64;
		
		// Conservative buffer distance that accounts for the size of each carved part.
		let buffer = (self.blob_size_factor * 2.0 + 16.0) as f64; 
		
		// Invalid: Subtraction from distance squared.
		self.position.distance_from_chunk_squared() - remaining * remaining > buffer * buffer
	}
	
	fn next_blob_size(&self, trig: &TrigLookup) -> BlobSize {
		BlobSize::sphere(MIN_H_SIZE + (trig.sin(self.size.current as f32 * NOTCH_PI / self.size.max as f32) * self.blob_size_factor) as f64)
	}
	
	pub fn step(&mut self, trig: &TrigLookup) -> Outcome {
		if self.size.done() {
			return Outcome::Done;
		}
		
		self.position.step(&mut self.state, trig, self.pitch_keep);
		
		if self.size.should_split(self.split) {
			return Outcome::Split;
		}
		
		if self.state.next_i32(4) == 0 {
			self.size.step();
			return Outcome::Constrict;
		}
		
		if self.is_chunk_unreachable() {
			return Outcome::Unreachable;
		}
		
		let size = self.next_blob_size(trig);
		
		if self.position.out_of_chunk(&size) {
			self.size.step();
			return Outcome::OutOfChunk;
		}
		
		let blob = self.position.blob(size);
		
		self.size.step();
		
		Outcome::Carve(blob)
	}
}

#[derive(Debug, Copy, Clone)]
struct SystemSize {
	current: i32,
	max:     i32
}

impl SystemSize {
	fn new(rng: &mut JavaRng, current: i32, max_chunk_radius: i32) -> Self {
		let max_block_radius = max_chunk_radius * 16 - 16;
		let max = max_block_radius - rng.next_i32(max_block_radius / 4);
		
		SystemSize { current, max }
	}
	
	pub fn step(&mut self) {
		self.current += 1;
	}
	
	pub fn done(&self) -> bool {
		self.current >= self.max
	}
	
	pub fn should_split(&self, split_threshold: Option<i32>) -> bool {
		Some(self.current) == split_threshold
	}
	
	/// Returns the point where the tunnel will split into 2. Returns None if it won't split.
	fn split(&self, rng: &mut JavaRng, blob_size_factor: f32) -> Option<i32> {
		let split = rng.next_i32(self.max / 2) + self.max / 4;
		
		if blob_size_factor > 1.0 {Some(split)} else {None}
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
	fn new(chunk: (i32, i32), block: (f64, f64, f64)) -> Self {
		Position {
			chunk,
			chunk_center: ((chunk.0 * 16 + 8) as f64, (chunk.1 * 16 + 8) as f64),
			block,
			yaw: 0.0,
			pitch: 0.0,
			yaw_velocity: 0.0,
			pitch_velocity: 0.0
		}
	}
	
	fn with_angles(chunk: (i32, i32), block: (f64, f64, f64), yaw: f32, pitch: f32) -> Self {
		Position {
			chunk,
			chunk_center: ((chunk.0 * 16 + 8) as f64, (chunk.1 * 16 + 8) as f64),
			block,
			yaw,
			pitch,
			yaw_velocity: 0.0,
			pitch_velocity: 0.0
		}
	}
	
	fn step(&mut self, rng: &mut JavaRng, trig: &TrigLookup, pitch_keep: f32) {
		let cos_pitch = trig.cos(self.pitch);
		
		self.block.0 += (trig.cos(self.yaw) * cos_pitch) as f64;
		self.block.1 +=  trig.sin(self.pitch)            as f64;
		self.block.2 += (trig.sin(self.yaw) * cos_pitch) as f64;
		
		self.pitch *= pitch_keep;
		self.pitch += self.pitch_velocity * 0.1;
		self.yaw += self.yaw_velocity * 0.1;
		
		self.pitch_velocity *= 0.9;
		self.yaw_velocity   *= 0.75;
		self.pitch_velocity += (rng.next_f32() - rng.next_f32()) * rng.next_f32() * 2.0;
		self.yaw_velocity   += (rng.next_f32() - rng.next_f32()) * rng.next_f32() * 4.0;
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
				min(max(lower.0, 0), 16)  as u8,
				min(max(lower.1, 1), 255) as u8,
				min(max(lower.2, 0), 16)  as u8
			),
			upper: (
				min(max(upper.0, 0), 16)  as u8,
				min(max(upper.1, 0), 120) as u8,
				min(max(upper.2, 0), 16)  as u8
			)
		}
	}
}

#[derive(Debug)]
enum Outcome {
	Split,
	Constrict,
	Unreachable,
	OutOfChunk,
	Carve(Blob),
	Done
}

impl Outcome {
	fn continues(&self) -> bool {
		match *self {
			Outcome::Split       => false,
			Outcome::Constrict   => true,
			Outcome::Unreachable => false,
			Outcome::OutOfChunk  => true,
			Outcome::Carve(_)    => true,
			Outcome::Done        => false
		}
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
	fn sphere(radius: f64) -> Self {
		BlobSize {
			horizontal: radius,
			vertical: radius
		}
	}
	
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

#[derive(Debug)]
struct Blob {
	/// Center of the blob
	center: (f64, f64, f64),
	/// Size of the blob
	size: BlobSize,
	/// Lower bounds of the feasible region, in chunk coordinates: [0,16), [0,128), [0,16)
	lower: (u8, u8, u8),
	/// Upper bounds of the feasible region, in chunk coordiantes: [0,16), [0,128), [0,16)
	upper: (u8, u8, u8)
}