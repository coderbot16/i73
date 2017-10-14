use chunk::storage::{Chunk, Target};
use chunk::grouping::Column;
use chunk::position::{LayerPosition, BlockPosition};
use dynamics::light::Lighting;

pub type ChunkMiniRegion<B> = MiniRegion<Chunk<B>>;
pub type LightMiniRegion<S> = MiniRegion<Lighting<S>>;

pub struct MiniRegion<T> where T: Clone {
	chunks: Box<[Option<T>]>
}

impl<T> MiniRegion<T> where T: Clone {
	pub fn new() -> Self {
		MiniRegion {
			chunks: vec![None; 4096].into_boxed_slice()
		}
	}
	
	pub fn add(&mut self, position: BlockPosition, chunk: T) {
		self.chunks[position.chunk_yzx() as usize] = Some(chunk);
	}
	
	pub fn remove(&mut self, position: BlockPosition) -> Option<T> {
		self.chunks[position.chunk_yzx() as usize].take()
	}
	
	pub fn get(&self, position: BlockPosition) -> Option<&T> {
		self.chunks[position.chunk_yzx() as usize].as_ref()
	}
	
	pub fn get_mut(&mut self, position: BlockPosition) -> Option<&mut T> {
		self.chunks[position.chunk_yzx() as usize].as_mut()
	}
}

impl<B> MiniRegion<Chunk<B>> where B: Target {
	pub fn set_columm(&mut self, position: LayerPosition, column: Column<B>) {
		let mut chunks = (Box::new(column.into_chunks()) as Box<[_]>).into_vec();
		
		for (index, chunk) in chunks.drain(..).enumerate() {
			let position = BlockPosition::new(position.x(), index as u8, position.z());
			
			self.add(position, chunk);
		}
	}
}