use chunk::storage::{Chunk, Target};
use chunk::grouping::Column;
use chunk::position::{LayerPosition, BlockPosition};
use dynamics::light::Lighting;
use std::collections::hash_map::{HashMap, Entry};

pub type ChunkMiniRegion<B> = MiniRegion<Chunk<B>>;
pub type LightMiniRegion<S> = MiniRegion<Lighting<S>>;

pub struct MiniRegion<T> where T: Clone {
	chunks: Box<[Option<T>]>,
	present: usize
}

impl<T> MiniRegion<T> where T: Clone {
	pub fn new() -> Self {
		MiniRegion {
			chunks: vec![None; 4096].into_boxed_slice(),
			present: 0
		}
	}
	
	pub fn add(&mut self, position: BlockPosition, chunk: T) {
		self.chunks[position.chunk_yzx() as usize] = Some(chunk);
		self.present += 1;
	}
	
	pub fn remove(&mut self, position: BlockPosition) -> Option<T> {
		let value = self.chunks[position.chunk_yzx() as usize].take();
		
		if value.is_some() {
			self.present -= 1;
		}
		
		value
	}
	
	pub fn get(&self, position: BlockPosition) -> Option<&T> {
		self.chunks[position.chunk_yzx() as usize].as_ref()
	}
	
	pub fn get_mut(&mut self, position: BlockPosition) -> Option<&mut T> {
		self.chunks[position.chunk_yzx() as usize].as_mut()
	}
	
	pub fn is_empty(&self) -> bool {
		self.present == 0
	}
}

impl<B> MiniRegion<Chunk<B>> where B: Target {
	pub fn set_column(&mut self, position: LayerPosition, column: Column<B>) {
		let mut chunks = (Box::new(column.into_chunks()) as Box<[_]>).into_vec();
		
		for (index, chunk) in chunks.drain(..).enumerate() {
			let position = BlockPosition::new(position.x(), index as u8, position.z());
			
			self.add(position, chunk);
		}
	}
}

pub struct World<T> where T: Clone {
	regions: HashMap<(i32, i32), MiniRegion<T>>
}

impl<T> World<T> where T: Clone {
	pub fn new() -> Self {
		World {
			regions: HashMap::new()
		}
	}
	
	pub fn add(&mut self, coords: (i32, u8, i32), chunk: T) {
		let (region, inner) = Self::split_coords(coords);
		
		self.regions.entry(region).or_insert(MiniRegion::new()).add(inner, chunk);
	}
	
	pub fn remove(&mut self, coords: (i32, u8, i32)) -> Option<T> {
		let (region, inner) = Self::split_coords(coords);
		
		if let Entry::Occupied(mut occupied) = self.regions.entry(region) {
			let value = occupied.get_mut().remove(inner);
			
			if occupied.get().is_empty() {
				occupied.remove();
			}
			
			value
		} else {
			None
		}
	}
	
	pub fn get(&mut self, coords: (i32, u8, i32)) -> Option<&T> {
		let (region, inner) = Self::split_coords(coords);
		
		self.regions.get(&region).and_then(|region| region.get(inner))
	}
	
	pub fn get_mut(&mut self, coords: (i32, u8, i32)) -> Option<&mut T> {
		let (region, inner) = Self::split_coords(coords);
		
		self.regions.get_mut(&region).and_then(|region| region.get_mut(inner))
	}
	
	pub fn into_regions(self) -> HashMap<(i32, i32), MiniRegion<T>> {
		self.regions
	}
	
	fn split_coords(coords: (i32, u8, i32)) -> ((i32, i32), BlockPosition) {
		let region = (coords.0 >> 4, coords.2 >> 4);
		let inner = BlockPosition::new((coords.0 & 15) as u8, coords.1, (coords.2 & 15) as u8);
		
		(region, inner)
	}
}

impl<B> World<Chunk<B>> where B: Target {
	pub fn set_column(&mut self, coords: (i32, i32), column: Column<B>) {
		let (region, inner) = Self::split_coords((coords.0, 0, coords.1));
		
		self.regions.entry(region).or_insert(MiniRegion::new()).set_column(inner.layer(), column)
	}
}