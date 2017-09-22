use chunk::anvil::NibbleVec;
use chunk::position::BlockPosition;
use std::hash::Hash;
use std::collections::HashMap;

pub struct Chunk<B> where B: Eq + Hash + Clone {
	storage: PackedBlockStorage,
	palette: Palette<B>
}

impl<B> Chunk<B> where B: Eq + Hash + Clone {
	pub fn new(bits_per_entry: usize) -> Self {
		Chunk {
			storage: PackedBlockStorage::new(bits_per_entry),
			palette: Palette::new(bits_per_entry)
		}
	}
	
	/// Finds a free entry in the entries table and adds the target, or reallocates if there are no free entries. 
	/// Additionally, if the target is already in the palette, it returns the index of that.
	fn ensure_available(&mut self, target: B) {
		unimplemented!()
	}
	
	// TODO: Methods to work with the palette: pruning, etc.
	
	pub fn palette_mut(&mut self) -> &mut Palette<B> {
		&mut self.palette
	}
	
	pub fn freeze_palette(&mut self) -> (&mut PackedBlockStorage, &Palette<B>) {
		(&mut self.storage, &self.palette)
	}
}

impl Chunk<u16> {
	/// Returns the Blocks, Metadata, and Add arrays for this chunk.
	/// Returns Err if unable to resolve an association.
	pub fn to_anvil(&self) -> Result<(Vec<i8>, NibbleVec, Option<NibbleVec>), usize> {
		let mut blocks = vec![0; 4096];
		let mut meta = NibbleVec::filled();
		
		let mut need_add = false;
		for entry in self.palette.entries.iter().filter_map(|&f| f) {
			// Can't express Anvil IDs over 4095 without Add.
			if entry > 4095 {
				need_add = true;
			}
		}
		
		if need_add {
			let mut add = NibbleVec::filled();
			
			for index in 0..4096 {
				let position = BlockPosition::from_yzx(index);
				let association = self.storage.get(position, &self.palette);
				let anvil = association.target().map(|&v| v)?;
				
				    blocks[index as usize] = (anvil >> 4)  as i8;
				meta.set_uncleared(position, (anvil & 0xF) as i8);
				 add.set_uncleared(position, (anvil >> 12) as i8);
			}
			
			Ok((blocks, meta, Some(add)))
		} else {
			for index in 0..4096 {
				let position = BlockPosition::from_yzx(index);
				let association = self.storage.get(position, &self.palette);
				let anvil = association.target().map(|&v| v)?;
				
				blocks[index as usize] = (anvil >> 4) as i8;
				meta.set_uncleared(position, (anvil & 0xF) as i8);
			}
			
			Ok((blocks, meta, None))
		}
	}
}

#[derive(Debug, Copy, Clone)]
pub struct PaletteAssociation<'p, B> where B: 'p + Eq + Hash + Clone {
	palette: &'p Palette<B>,
	value: usize
}

impl<'p, B> PaletteAssociation<'p, B> where B: 'p + Eq + Hash + Clone {
	pub fn target(&self) -> Result<&B, usize> {
		self.palette.entries[self.value].as_ref().ok_or(self.value)
	}
	
	pub fn raw_value(&self) -> usize {
		self.value
	}
}

#[derive(Debug)]
pub struct Palette<B> where B: Eq + Hash + Clone {
	entries: Vec<Option<B>>,
	reverse: HashMap<B, usize>
}

impl<B> Palette<B> where B: Eq + Hash + Clone {
	pub fn new(bits_per_entry: usize) -> Self {
		Palette {
			entries: vec![None; 1<<bits_per_entry],
			reverse: HashMap::new()
		}
	}
	
	/// Replaces the entry at `index` with the target, even if `index` was previously vacant. 
	pub fn replace(&mut self, index: usize, target: B) {
		self.entries[index] = Some(target.clone());
		// Only replace entries in the reverse lookup if they don't exist, otherwise keep the previous entry.
		self.reverse.entry(target).or_insert(index);
	}
	
	/// Gets an association that will reference back to the target. Note that several indices may point to the same target, this returns one of them.
	pub fn reverse_lookup<'p>(&'p self, target: &B) -> Option<PaletteAssociation<'p, B>> {
		self.reverse.get(target).map(|&value| PaletteAssociation { palette: self, value })
	}
}

pub struct PackedBlockStorage {
	storage: Vec<u64>,
	counts: Vec<usize>,
	bits_per_entry: usize,
	bitmask: u64
}

enum Indices {
	Single(usize),
	Double(usize, usize)
}

impl PackedBlockStorage {
	pub fn new(bits_per_entry: usize) -> Self {
		let mut counts = vec![0; 1<<bits_per_entry];
		counts[0] = 4096;
		
		PackedBlockStorage {
			storage: vec![0; bits_per_entry * 512],
			counts,
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
	
	pub fn get_count<B>(&self, association: &PaletteAssociation<B>) -> usize where B: Eq + Hash + Clone {
		self.counts[association.raw_value()]
	}
	
	pub fn get<'p, B>(&self, position: BlockPosition, palette: &'p Palette<B>) -> PaletteAssociation<'p, B> where B: 'p + Eq + Hash + Clone {
		if self.bits_per_entry == 0 {
			return PaletteAssociation {
				palette,
				value: 0
			}
		}
		
		let index = position.chunk_yzx() as usize;
		
		let (indices, sub_index) = self.indices(index);
		
		let raw = (match indices {
			Indices::Single(index) => self.storage[index] >> sub_index,
			Indices::Double(start, end) => {
				let end_sub_index = 64 - sub_index;
				(self.storage[start] >> sub_index) | (self.storage[end] << end_sub_index)
			}
		} & self.bitmask);
		
		PaletteAssociation {
			palette,
			value: raw as usize
		}
	}
	
	pub fn set<'p, B>(&mut self, position: BlockPosition, association: &PaletteAssociation<'p, B>) where B: 'p + Eq + Hash + Clone {
		if self.bits_per_entry == 0 {
			return;
		}
		
		let value = association.value as u64;
		let index = position.chunk_yzx() as usize;
		
		let previous = self.get(position, association.palette);
		self.counts[previous.raw_value()] -= 1;
		self.counts[association.raw_value()] += 1;
		
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