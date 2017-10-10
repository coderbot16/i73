use chunk::anvil::NibbleVec;
use chunk::position::BlockPosition;
use std::hash::Hash;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::mem;
use std::marker::PhantomData;
use std::fmt::Debug;

pub trait Target: Eq + Hash + Clone + Debug {}
impl<T> Target for T where T: Eq + Hash + Clone + Debug {}

#[derive(Debug)]
pub struct Chunk<B> where B: Target {
	storage: PackedBlockStorage<BlockPosition>,
	palette: Palette<B>
}

impl<B> Chunk<B> where B: Target {
	pub fn new(bits_per_entry: usize) -> Self {
		Chunk {
			storage: PackedBlockStorage::new(bits_per_entry),
			palette: Palette::new(bits_per_entry)
		}
	}
	
	/// Increased the capacity of this chunk's storage by 1 bit, and returns the old storage for reuse purposes.
	pub fn reserve_bits(&mut self, bits: usize) -> PackedBlockStorage<BlockPosition> {
		self.palette.reserve_bits(bits);
		
		let mut replacement_storage = PackedBlockStorage::new(self.storage.bits_per_entry() + bits);
		replacement_storage.clone_from(&self.storage, &self.palette);
		
		mem::swap(&mut self.storage, &mut replacement_storage);
		
		replacement_storage
	}
	
	/// Makes sure that a future lookup for the target will succeed, unless the entry has changed since this call.
	pub fn ensure_available(&mut self, target: B) {
		 if let Err(target) = self.palette.try_insert(target) {
		 	self.reserve_bits(1);
		 	self.palette.try_insert(target).expect("There should be room for a new entry, we just made some!");
		 }
	}
	
	pub fn get(&self, position: BlockPosition) -> PaletteAssociation<B> {
		self.storage.get(position, &self.palette)
	}
	
	// TODO: Methods to work with the palette: pruning, etc.
	
	pub fn palette_mut(&mut self) -> &mut Palette<B> {
		&mut self.palette
	}
	
	pub fn freeze_read_only(&self) -> (&PackedBlockStorage<BlockPosition>, &Palette<B>) {
		(&self.storage, &self.palette)
	}
	
	pub fn freeze_palette(&mut self) -> (&mut PackedBlockStorage<BlockPosition>, &Palette<B>) {
		(&mut self.storage, &self.palette)
	}
	
	/// Preforms the ensure_available, reverse_lookup, and set calls all in one.
	/// Prefer freezing the palette for larger scale block sets.
	pub fn set_immediate(&mut self, position: BlockPosition, target: &B) {
		self.ensure_available(target.clone());
		let association = self.palette.reverse_lookup(&target).unwrap();
		
		self.storage.set(position, &association);
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
			// Can't express Anvil IDs over 4095 without Add. TODO: Utilize Counts.
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
				meta.set_uncleared(position, (anvil & 0xF) as u8);
				 add.set_uncleared(position, (anvil >> 12) as u8);
			}
			
			Ok((blocks, meta, Some(add)))
		} else {
			for index in 0..4096 {
				let position = BlockPosition::from_yzx(index);
				let association = self.storage.get(position, &self.palette);
				let anvil = association.target().map(|&v| v)?;
				
				    blocks[index as usize] = (anvil >> 4)  as i8;
				meta.set_uncleared(position, (anvil & 0xF) as u8);
			}
			
			Ok((blocks, meta, None))
		}
	}
}

#[derive(Debug, Copy, Clone)]
pub struct PaletteAssociation<'p, B> where B: 'p + Target {
	palette: &'p Palette<B>,
	value: usize
}

impl<'p, B> PaletteAssociation<'p, B> where B: 'p + Target {
	pub fn target(&self) -> Result<&B, usize> {
		self.palette.entries[self.value].as_ref().ok_or(self.value)
	}
	
	pub fn raw_value(&self) -> usize {
		self.value
	}
}

#[derive(Debug)]
pub struct Palette<B> where B: Target {
	entries: Vec<Option<B>>,
	reverse: HashMap<B, usize>
}

impl<B> Palette<B> where B: Target {
	pub fn new(bits_per_entry: usize) -> Self {
		Palette {
			entries: vec![None; 1<<bits_per_entry],
			reverse: HashMap::new()
		}
	}
	
	pub fn reserve_bits(&mut self, bits: usize) {
		for _ in 0..bits {
			let additional = self.entries.len();
			self.entries.reserve(additional);
			
			for _ in 0..additional {
				self.entries.push(None);
			}
		}
	}
	
	pub fn try_insert(&mut self, target: B) -> Result<usize, B> {
		match self.reverse.entry(target.clone()) {
			Entry::Occupied(occupied) => Ok(*occupied.get()),
			Entry::Vacant(vacant) => {
				let mut idx = None;
				for (index, slot) in self.entries.iter_mut().enumerate() {
					if slot.is_none() {
						*slot = Some(target);
						idx = Some(index);
						break;
					}
				}
				
				match idx {
					Some(index) => {
						vacant.insert(index);
						Ok(index)
					},
					None => Err(vacant.into_key())
				}
			}
		}
	}
	
	/// Replaces the entry at `index` with the target, even if `index` was previously vacant. 
	pub fn replace(&mut self, index: usize, target: B) {
		let mut old = Some(target.clone());
		mem::swap(&mut old, &mut self.entries[index]);
		
		if let Some(old_target) = old {
			let mut other_reference = None;
		
			for (index, entry) in self.entries.iter().enumerate() {
				if let &Some(ref other) = entry {
					if *other == old_target {
						other_reference = Some(index);
						break;
					}
				}
			}
			
			if let Entry::Occupied(mut occ) = self.reverse.entry(old_target) {
				if let Some(other) = other_reference {
					if *occ.get() == index {
						occ.insert(other);
					}
				} else {
					occ.remove();
				}
			}
		}
		
		// Only replace entries in the reverse lookup if they don't exist, otherwise keep the previous entry.
		self.reverse.entry(target).or_insert(index);
	}
	
	/// Gets an association that will reference back to the target. Note that several indices may point to the same target, this returns one of them.
	pub fn reverse_lookup(&self, target: &B) -> Option<PaletteAssociation<B>> {
		self.reverse.get(target).map(|&value| PaletteAssociation { palette: self, value })
	}
	
	pub fn entries(&self) -> &[Option<B>] {
		&self.entries
	}
}

pub trait PackedIndex: Copy {
	fn entries() -> usize;
	fn from_index(index: usize) -> Self;
	fn to_index(&self) -> usize;
}

#[derive(Debug)]
pub struct PackedBlockStorage<P> where P: PackedIndex {
	storage: Vec<u64>,
	counts: Vec<usize>,
	bits_per_entry: usize,
	bitmask: u64,
	phantom: PhantomData<P>
}

enum Indices {
	Single(usize),
	Double(usize, usize)
}

impl<P> PackedBlockStorage<P> where P: PackedIndex {
	pub fn new(bits_per_entry: usize) -> Self {
		let mut counts = vec![0; 1<<bits_per_entry];
		counts[0] = P::entries();
		
		PackedBlockStorage {
			storage: vec![0; bits_per_entry * (P::entries() / 64)],
			counts,
			bits_per_entry,
			bitmask: (1 << (bits_per_entry as u64)) - 1,
			phantom: PhantomData
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
	
	pub fn get_count<B>(&self, association: &PaletteAssociation<B>) -> usize where B: Target {
		self.counts[association.raw_value()]
	}
	
	pub fn get<'p, B>(&self, position: P, palette: &'p Palette<B>) -> PaletteAssociation<'p, B> where B: 'p + Target {
		if self.bits_per_entry == 0 {
			return PaletteAssociation {
				palette,
				value: 0
			}
		}
		
		let index = position.to_index();
		
		let (indices, sub_index) = self.indices(index);
		
		let raw = match indices {
			Indices::Single(index) => self.storage[index] >> sub_index,
			Indices::Double(start, end) => {
				let end_sub_index = 64 - sub_index;
				(self.storage[start] >> sub_index) | (self.storage[end] << end_sub_index)
			}
		} & self.bitmask;
		
		PaletteAssociation {
			palette,
			value: raw as usize
		}
	}
	
	pub fn set<B>(&mut self, position: P, association: &PaletteAssociation<B>) where B: Target {
		if self.bits_per_entry == 0 {
			return;
		}
		
		let value = association.value as u64;
		let index = position.to_index();
		
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
	
	pub fn clone_from<B>(&mut self, from: &PackedBlockStorage<P>, palette: &Palette<B>) -> bool where B: Target {
		if from.bits_per_entry < self.bits_per_entry {
			return false;
		}
		
		let added_bits = from.bits_per_entry - self.bits_per_entry;
		
		self.counts.clear();
		
		for count in &from.counts {
			self.counts.push(*count);
		}
		
		for _ in 0..added_bits {
			let add = self.counts.len();
			self.counts.reserve(add);
			
			for _ in 0..add {
				self.counts.push(0);
			}
		}
		
		if added_bits == 0 {
			self.storage.clone_from(&from.storage);
		} else {
			// TODO: Optimize this loop!
			
			for index in 0..P::entries() {
				let position = P::from_index(index);
				self.set(position, &from.get(position, palette));
			}
		}
		
		true
	}
	
	pub fn bits_per_entry(&self) -> usize {
		self.bits_per_entry
	}
}