use chunk::position::{BlockPosition, LayerPosition};
use std::collections::{HashMap, HashSet};
use std::mem;

/// Alternative to a recursive lighting algorithm. Also much faster and more efficient.
pub struct Queue {
	primary:   Box<[u64; 64]>,
	secondary: Box<[u64; 64]>,
	skip:      usize
}

impl Queue {
	pub fn new() -> Self {
		Queue {
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
		
		self.skip = usize::max_value();
		
		false
	}
	
	pub fn next(&mut self) -> BlockPosition {
		let block = self.primary[self.skip];
		let index = (self.skip * 64) | (block.trailing_zeros() as usize);
		
		BlockPosition::from_yzx(index as u16)
	}
	
	pub fn flip(&mut self) -> bool {
		self.skip = 0;
		mem::swap(&mut self.primary, &mut self.secondary);
		self.fast_forward()
	}
	
	pub fn dequeue(&mut self, position: BlockPosition) -> bool {
		let index = position.chunk_yzx() as usize;
		
		self.primary[index / 64] &= !(1 << (index % 64));
		self.fast_forward()
	}
	
	pub fn enqueue(&mut self, position: BlockPosition) {
		let index = position.chunk_yzx() as usize;
		
		self.secondary[index / 64] |= 1 << (index % 64);
	}
	
	pub fn enqueue_neighbors(&mut self, position: BlockPosition) {
		position.minus_x().map(|at| self.enqueue(at));
		position. plus_x().map(|at| self.enqueue(at));
		position.minus_z().map(|at| self.enqueue(at));
		position. plus_z().map(|at| self.enqueue(at));
		position.minus_y().map(|at| self.enqueue(at));
		position. plus_y().map(|at| self.enqueue(at));
	}
}

pub struct WorldQueue {
	regions: HashMap<(i32, i32), Queue>
}

impl WorldQueue {
	pub fn new() -> Self {
		WorldQueue {
			regions: HashMap::new()
		}
	}
	
	pub fn clear(&mut self) {
		self.regions.clear();
	}
	
	pub fn flip(&mut self) -> bool {
		let mut removals = HashSet::new();
		
		for (region, queue) in self.regions.iter_mut() {
			if !queue.flip() {
				removals.insert(*region);
			}
		}
		
		for removal in removals {
			self.regions.remove(&removal);
		}
		
		!self.regions.is_empty()
	}
	
	pub fn dequeue_next(&mut self) -> Option<(i32, u8, i32)> {
		// TODO: Better iteration algorithm.
		
		for (&region, queue) in self.regions.iter_mut() {
			if queue.skip != usize::max_value() {
				let inner = queue.next();
				queue.dequeue(inner);
				
				return Some((region.0 << 4 | (inner.x() as i32), inner.y(), region.1 << 4 | (inner.z() as i32)))
			}
		}
		
		None
	}
	
	pub fn enqueue(&mut self, coords: (i32, u8, i32)) {
		let (region, inner) = Self::split_coords(coords);
		
		self.regions.entry(region).or_insert(Queue::new()).enqueue(inner);
	}
	
	pub fn enqueue_neighbors(&mut self, coords: (i32, u8, i32)) {
		self.enqueue((coords.0 + 1, coords.1,     coords.2    ));
		self.enqueue((coords.0 - 1, coords.1,     coords.2    ));
		self.enqueue((coords.0,     coords.1,     coords.2 + 1));
		self.enqueue((coords.0,     coords.1,     coords.2 - 1));
		
		if coords.1 < 255 {
			self.enqueue((coords.0,     coords.1 + 1, coords.2    ));
		}
		
		if coords.1 > 0 {
			self.enqueue((coords.0,     coords.1 -1, coords.2    ));
		}
	}
	
	fn split_coords(coords: (i32, u8, i32)) -> ((i32, i32), BlockPosition) {
		let region = (coords.0 >> 4, coords.2 >> 4);
		let inner = BlockPosition::new((coords.0 & 15) as u8, coords.1, (coords.2 & 15) as u8);
		
		(region, inner)
	}
}

#[derive(Debug, Default)]
pub struct LayerMask([u64; 4]);
impl LayerMask {
	pub fn set_or(&mut self, position: LayerPosition, value: bool) {
		let index = position.zx();
		
		let array_index = (index / 64) as usize;
		let shift = index % 64;
		
		self.0[array_index] |= (value as u64) << shift
	}
	
	pub fn set(&mut self, position: LayerPosition, value: bool) {
		let index = position.zx();
		
		let array_index = (index / 64) as usize;
		let shift = index % 64;
		
		let cleared = self.0[array_index] & !(1 << shift);
		self.0[array_index] = cleared | ((value as u64) << shift)
	}
	
	pub fn get(&self, position: LayerPosition) -> bool {
		let index = position.zx();
		
		let array_index = (index / 64) as usize;
		
		(self.0[array_index] >> ((index % 64) as usize) & 1) == 1
	}
	
	pub fn set_all(&mut self, value: bool) {
		let value = if value {u64::max_value()} else {0};
		
		self.0[0] = value;
		self.0[1] = value;
		self.0[2] = value;
		self.0[3] = value;
	}
	
	pub fn any(&self) -> bool {
		(self.0[0] | self.0[1] | self.0[2] | self.0[3]) != 0
	}
	
	pub fn all(&self) -> bool {
		(self.0[0] & self.0[1] & self.0[2] & self.0[3]) == u64::max_value()
	}
}