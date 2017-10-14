use chunk::storage::{Chunk, Target};
use chunk::grouping::Column;
use chunk::position::BlockPosition;
use chunk::anvil::NibbleVec;
use std::mem;
use std::cmp::max;

type Unpacked = u8;
type Packed = u8;

// TODO: Generate from opacity rather than block type.
pub fn generate_heightmap<B>(column: &Column<B>, target: &B) -> [u32; 256] where B: Target {
	let mut heightmap = [0; 256];
	
	for (xz, height) in heightmap.iter_mut().enumerate() {
		let mut position = BlockPosition::from_yzx(0xFF00 | (xz as u16));
		
		loop {
			if column.get(position).target() != Ok(target) {
				*height = (position.y() as u32) + 1;
				break;
			}
			
			match position.minus_y() {
				Some(below) => position = below,
				None => break 
			}
		}
	}
	
	heightmap
}

/// Alternative to a recursive lighting algorithm. Also much faster and more efficient.
pub struct LightingQueue {
	primary:   Box<[u64; 64]>,
	secondary: Box<[u64; 64]>,
	skip:      usize
}

impl LightingQueue {
	pub fn new() -> Self {
		LightingQueue {
			primary:   Box::new([0; 64]),
			secondary: Box::new([0; 64]),
			skip:      0
		}
	}
	
	pub fn clear(&mut self) {
		self.skip = 0;
		
		for value in self.primary.iter_mut() {
			*value = 0;
		}
		
		for value in self.secondary.iter_mut() {
			*value = 0;
		}
	}
	
	fn fast_forward(&mut self) -> bool {
		for (index, block) in (&self.primary[self.skip..]).iter().enumerate() {
			if *block != 0 {
				self.skip += index;
				return true;
			}
		}
		
		false
	}
	
	fn next(&mut self) -> BlockPosition {
		let block = self.primary[self.skip];
		let index = (self.skip * 64) | (block.trailing_zeros() as usize);
		
		BlockPosition::from_yzx(index as u16)
	}
	
	fn flip(&mut self) -> bool {
		self.skip = 0;
		mem::swap(&mut self.primary, &mut self.secondary);
		self.fast_forward()
	}
	
	fn dequeue(&mut self, position: BlockPosition) -> bool {
		let index = position.chunk_yzx() as usize;
		
		self.primary[index / 64] &= !(1 << (index % 64));
		self.fast_forward()
	}
	
	fn enqueue(&mut self, position: BlockPosition) {
		let index = position.chunk_yzx() as usize;
		
		self.secondary[index / 64] |= 1 << (index % 64);
	}
	
	fn enqueue_neighbors(&mut self, position: BlockPosition) {
		position.minus_x().map(|at| self.enqueue(at));
		position. plus_x().map(|at| self.enqueue(at));
		position.minus_z().map(|at| self.enqueue(at));
		position. plus_z().map(|at| self.enqueue(at));
		position.minus_y().map(|at| self.enqueue(at));
		position. plus_y().map(|at| self.enqueue(at));
	}
}

pub struct LightingData(Box<[Packed; 2048]>);
impl LightingData {
	pub fn new() -> Self {
		LightingData(Box::new([0; 2048]))
	}
	
	fn set_raw(&mut self, at: BlockPosition, light: Unpacked) {
		let light = light & 15;
		
		let (index, shift) = at.chunk_nibble_yzx();
		let cleared = !((!self.0[index]) | (0xF << shift));
		self.0[index] = cleared | (light << shift);
	}
	
	pub fn set(&mut self, queue: &mut LightingQueue, at: BlockPosition, light: Unpacked) {
		self.set_raw(at, light);
		queue.enqueue_neighbors(at);
	}
	
	pub fn get(&self, at: BlockPosition) -> Unpacked {
		let (index, shift) = at.chunk_nibble_yzx();
		(self.0[index]&(0xF << shift)) >> shift
	}
	
	pub fn to_anvil(self) -> NibbleVec {
		NibbleVec::from_vec((self.0 as Box<[u8]>).into_vec()).unwrap()
	}
}

impl ::std::fmt::Debug for LightingData {
	fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
		write!(f, "{:?}", &self.0[..])
	}
}

#[derive(Debug, Default)]
pub struct LayerMask([u64; 4]);
impl LayerMask {
	pub fn set(&mut self, index: u8, value: bool) {
		let array_index = (index / 64) as usize;
		let shift = index % 64;
		
		let cleared = self.0[array_index] & !(1 << shift);
		self.0[array_index] = cleared | ((value as u64) << shift)
	}
	
	pub fn get(&self, index: u8) -> bool {
		let array_index = (index / 64) as usize;
		
		(self.0[array_index] >> ((index % 64) as usize) & 1) == 1
	}
	
	pub fn any(&self) -> bool {
		(self.0[0] | self.0[1] | self.0[2] | self.0[3]) > 0
	}
}

pub struct Lighting<S> where S: LightSources {
	data: LightingData,
	emit: Box<[[Packed; 128]; 6]>,
	sources: S,
	meta: Vec<Meta>
}

impl<S> Lighting<S> where S: LightSources {
	pub fn new(sources: S, meta: Vec<Meta>) -> Self {
		Lighting {
			data: LightingData::new(),
			emit: Box::new([[0; 128]; 6]),
			sources,
			meta
		}
	}
	
	fn set_raw(&mut self, at: BlockPosition, light: Unpacked) {
		self.data.set_raw(at, light)
	}
	
	pub fn set(&mut self, queue: &mut LightingQueue, at: BlockPosition, light: Unpacked) {
		self.data.set(queue, at, light)
	}
	
	pub fn get(&self, at: BlockPosition) -> Unpacked {
		self.data.get(at)
	}
	
	pub fn initial<B>(&mut self, chunk: &Chunk<B>, queue: &mut LightingQueue) where B: Target {
		self.sources.initial(chunk, &mut self.data, queue)
	}
	
	pub fn step<B>(&mut self, chunk: &Chunk<B>, queue: &mut LightingQueue) -> bool where B: Target {
		if !queue.flip() {
			return false;
		}
		
		loop {
			let at = queue.next();
			
			let max_value = max(
				max(
					max(
						at.minus_x().map(|at| self.get(at)).unwrap_or(0), 
						at. plus_x().map(|at| self.get(at)).unwrap_or(0)
					),
					max(
						at.minus_z().map(|at| self.get(at)).unwrap_or(0), 
						at. plus_z().map(|at| self.get(at)).unwrap_or(0)
					)
				),
				max(
					at.minus_y().map(|at| self.get(at)).unwrap_or(0), 
					at. plus_y().map(|at| self.get(at)).unwrap_or(0)
				)
			);
			
			let new_light = max(max_value.saturating_sub(1), self.sources.emission(chunk, at)).saturating_sub(self.meta[chunk.get(at).raw_value()].opacity());
			
			if new_light != self.get(at) {
				self.set(queue, at, new_light);
			}
			
			if !queue.dequeue(at) {
				return true;
			}
		}
	}
	
	pub fn finish<B>(&mut self, chunk: &Chunk<B>, queue: &mut LightingQueue) -> usize where B: Target {
		let mut iterations = 0;
		
		while self.step(chunk, queue) { iterations += 1 };
		
		iterations
	}
	
	pub fn to_anvil(self) -> NibbleVec {
		self.data.to_anvil()
	}
	
	pub fn decompose(self) -> (LightingData, S) {
		(self.data, self.sources)
	}
	
	pub fn meta(&self) -> &[Meta] {
		&self.meta
	}
}

impl<S> ::std::fmt::Debug for Lighting<S> where S: LightSources + ::std::fmt::Debug {
	fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
		write!(f, "Lighting {{ data: {:?}, emit: [", self.data)?;
		
		for (index, emit) in self.emit.iter().enumerate() {
			write!(f, "{:?}", &emit[..])?;
			if index != 5 {
				write!(f, ",")?;
			}
		}
		
		write!(f, "], sources: {:?}, meta: {:?} }}", &self.sources, &self.meta[..])
	}
}

// TODO: More advanced directional lighting system.

#[derive(Copy, Clone, Debug)]
pub struct Meta(Unpacked);

impl Meta {
	pub fn new(opacity: Unpacked) -> Self {
		Meta(opacity)
	}
	
	pub fn opacity(&self) -> Unpacked {
		self.0 & 15
	}
}

impl Default for Meta {
	fn default() -> Self {
		Meta(0)
	}
}

pub trait LightSources {
	fn emission<B>(&self, chunk: &Chunk<B>, position: BlockPosition) -> Unpacked where B: Target;
	fn initial<B>(&self, chunk: &Chunk<B>, data: &mut LightingData, queue: &mut LightingQueue) where B: Target {
		unimplemented!()
	}
}

#[derive(Debug)]
pub struct BlockLightSources {
	emission: Vec<Unpacked>
}

impl BlockLightSources {
	pub fn new(palette_bits: usize) -> Self {
		BlockLightSources {
			emission: vec![0; 1 << palette_bits]
		}
	}
	
	pub fn set_emission(&mut self, raw_index: usize, value: Unpacked) {
		self.emission[raw_index] = value;
	}
}

impl LightSources for BlockLightSources {
	fn emission<B>(&self, chunk: &Chunk<B>, position: BlockPosition) -> Unpacked where B: Target {
		self.emission[chunk.get(position).raw_value()]
	}
}

pub struct SkyLightSources {
	heightmap: [u8; 128],
	no_light:  LayerMask
}

impl SkyLightSources {
	pub fn build<B>(chunk: &Chunk<B>, meta: &[Meta], mut no_light: LayerMask) -> Self where B: Target {
		for z in 0..16 {
			for x in 0..16 {
				let position = BlockPosition::new(x, 15, z);
				
				if meta[chunk.get(position).raw_value()].opacity() > 0 {
					no_light.set(z * 16 + x, true)
				}
			}
		}
		
		let mut heightmap = [0; 128];
		
		for z in 0..16 {
			for x in 0..16 {
				for y in (0..15).rev() {
					let position = BlockPosition::new(x, y, z);
				
					if meta[chunk.get(position).raw_value()].opacity() > 0 {
						let shift = position.chunk_nibble_yzx().1 as u8;
						let index = (z * 16 + x) as usize / 2;
						
						heightmap[index] |= (y + 1) << shift;
						
						break;
					}
				}
				
			}
		}
		
		SkyLightSources {
			heightmap,
			no_light
		}
	}
	
	pub fn height(&self, index: u8) -> u8 {
		let shift = (index & 1) as u8 * 4;
		
		(self.heightmap[(index / 2) as usize] >> shift) & 0xF
	}
	
	pub fn into_mask(self) -> LayerMask {
		self.no_light
	}
}

impl LightSources for SkyLightSources {
	fn emission<B>(&self, chunk: &Chunk<B>, position: BlockPosition) -> Unpacked where B: Target {
		if !self.no_light.get(position.zx()) {
			if position.y() >= self.height(position.zx()) { 15 } else { 0 }
		} else {
			0
		}
	}
	
	fn initial<B>(&self, chunk: &Chunk<B>, data: &mut LightingData, queue: &mut LightingQueue) where B: Target {
		// TODO: Check if initial lighting is unneccesary or can be skipped in some way.
		
		for z in 0..16 {
			for x in 0..16 {
				if self.no_light.get(z * 16 + x) {
					continue;
				}
				
				for y in (self.height(z * 16 + x)..16).rev() {
					data.set(queue, BlockPosition::new(x, y, z), 15);
				}
			}
		}
	}
}

impl ::std::fmt::Debug for SkyLightSources {
	fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
		write!(f, "SkyLightSources {{ heightmap: {:?}, no_light: {:?} }}", &self.heightmap[..], self.no_light)
	}
}