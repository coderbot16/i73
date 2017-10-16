use chunk::storage::{Chunk, Target};
use chunk::grouping::Column;
use chunk::position::BlockPosition;
use chunk::anvil::NibbleVec;
use dynamics::queue::{Queue, LayerMask};
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
	
	pub fn set(&mut self, queue: &mut Queue, at: BlockPosition, light: Unpacked) {
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
	
	pub fn set(&mut self, queue: &mut Queue, at: BlockPosition, light: Unpacked) {
		if light != self.get(at) {
			self.data.set(queue, at, light)
		}
	}
	
	pub fn get(&self, at: BlockPosition) -> Unpacked {
		self.data.get(at)
	}
	
	pub fn initial<B>(&mut self, chunk: &Chunk<B>, queue: &mut Queue) where B: Target {
		self.sources.initial(chunk, &mut self.data, queue)
	}
	
	pub fn step<B>(&mut self, chunk: &Chunk<B>, queue: &mut Queue) -> bool where B: Target {
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
			
			self.set(queue, at, new_light);
			
			if !queue.dequeue(at) {
				return true;
			}
		}
	}
	
	pub fn finish<B>(&mut self, chunk: &Chunk<B>, queue: &mut Queue) -> usize where B: Target {
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
	fn initial<B>(&self, chunk: &Chunk<B>, data: &mut LightingData, queue: &mut Queue) where B: Target;
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
	
	fn initial<B>(&self, _chunk: &Chunk<B>, _data: &mut LightingData, _queue: &mut Queue) where B: Target {
		unimplemented!()
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
				
				no_light.set_or(z * 16 + x, meta[chunk.get(position).raw_value()].opacity() > 0);
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
	
	pub fn into_mask(mut self) -> LayerMask {
		for z in 0..16 {
			for x in 0..16 {
				let height = self.height(z * 16 + x);
				
				self.no_light.set_or(z * 16 + x, height > 0);
			}
		}
		
		self.no_light
	}
}

impl LightSources for SkyLightSources {
	fn emission<B>(&self, _: &Chunk<B>, position: BlockPosition) -> Unpacked where B: Target {
		if !self.no_light.get(position.zx()) {
			if position.y() >= self.height(position.zx()) { 15 } else { 0 }
		} else {
			0
		}
	}
	
	fn initial<B>(&self, _: &Chunk<B>, data: &mut LightingData, queue: &mut Queue) where B: Target {
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