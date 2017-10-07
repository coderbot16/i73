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
		unimplemented!()
		/*self.primary.clear();
		self.secondary.clear();*/
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

pub struct Lighting<S> where S: LightSources {
	data: Box<[Packed; 2048]>,
	emit: Box<[[Packed; 128]; 6]>,
	sources: S,
	meta: Vec<Meta>
}

impl<S> Lighting<S> where S: LightSources {
	pub fn new(sources: S, palette_bits: usize) -> Self {
		Lighting {
			data: Box::new([0; 2048]),
			emit: Box::new([[0; 128]; 6]),
			sources,
			meta: vec![Meta::default(); 1 << palette_bits]
		}
	}
	
	fn set_raw(&mut self, at: BlockPosition, light: Unpacked) {
		let light = light & 15;
		
		let (index, shift) = at.chunk_nibble_yzx();
		let cleared = !((!self.data[index]) | (0xF << shift));
		self.data[index] = cleared | (light << shift);
	}
	
	pub fn set(&mut self, queue: &mut LightingQueue, at: BlockPosition, light: Unpacked) {
		self.set_raw(at, light);
		queue.enqueue_neighbors(at);
	}
	
	pub fn get(&self, at: BlockPosition) -> Unpacked {
		let (index, shift) = at.chunk_nibble_yzx();
		(self.data[index]&(0xF << shift)) >> shift
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
			
			// TODO: Opacity.
			
			let new_light = max(max_value.saturating_sub(1), self.sources.emission(chunk, at));
			
			if new_light != self.get(at) {
				self.set(queue, at, new_light);
			}
			
			if !queue.dequeue(at) {
				return true;
			}
		}
	}
	
	pub fn to_anvil(self) -> NibbleVec {
		NibbleVec::from_vec((self.data as Box<[u8]>).into_vec()).unwrap()
	}
}

impl<S> ::std::fmt::Debug for Lighting<S> where S: LightSources + ::std::fmt::Debug {
	fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
		write!(f, "Lighting {{ data: {:?}, emit: [", &self.data[..])?;
		
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
struct Meta(Unpacked);

impl Meta {
	fn new(opacity: Unpacked) -> Self {
		Meta(opacity)
	}
	
	fn opacity(&self) -> Unpacked {
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