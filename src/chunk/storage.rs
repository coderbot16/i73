use chunk::position::BlockPosition;

struct Column(Vec<Option<Chunk>>);

struct Chunk {
	storage: PackedBlockStorage,
	palette: () // TODO
}

struct PackedBlockStorage {
	storage: Vec<u64>,
	bits_per_entry: usize,
	bitmask: u64
}

enum Indices {
	Single(usize),
	Double(usize, usize)
}

impl PackedBlockStorage {
	pub fn new(bits_per_entry: usize) -> Self {
		PackedBlockStorage {
			storage: Vec::with_capacity(bits_per_entry * 512),
			bits_per_entry,
			bitmask: (1 << (bits_per_entry as u64)) - 1
		}
	}
	
	fn indices(&self, index: usize) -> (Indices, u8) {
		let bit_index = index*self.bits_per_entry;
		// Calculate the indices to the u64 array.
		let start = bit_index / 64;
		let end = ((bit_index + self.bits_per_entry) - 1) / 64;
		let sub_index = (bit_index % 64) as u8;
		
		// Does the packed sample start and end in the same u64?
		if start==end {
			(Indices::Single(start), sub_index)
		} else {
			(Indices::Double(start, end), sub_index)
		}
	}
	
	pub fn get(&self, position: BlockPosition) -> u64 {
		let index = position.yzx() as usize;
		
		let (indices, sub_index) = self.indices(index);
		
		(match indices {
			Indices::Single(index) => self.storage[index] >> sub_index,
			Indices::Double(start, end) => {
				let end_sub_index = 64 - sub_index;
				(self.storage[start] >> sub_index) | (self.storage[end] << end_sub_index)
			}
		} & self.bitmask)
	}
	
	pub fn set(&mut self, position: BlockPosition, value: u64) {
		let index = position.yzx() as usize;
		
		let (indices, sub_index) = self.indices(index);
		match indices {
			Indices::Single(index) => self.storage[index] = self.storage[index] & !(self.bitmask << sub_index) | (value & self.bitmask) << sub_index,
			Indices::Double(start, end) => {
				let end_sub_index = 64 - sub_index;
				self.storage[start] = self.storage[start] & !(self.bitmask << sub_index)  | (value & self.bitmask) << sub_index;
				self.storage[end]   = self.storage[end] >> end_sub_index << end_sub_index | (value & self.bitmask) >> end_sub_index;
			}
		}
	}
}