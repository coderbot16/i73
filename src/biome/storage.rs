use vocs::world::chunk::{Palette, PaletteAssociation, Target, NullRecorder};
use vocs::storage::packed::PackedBlockStorage;
use vocs::position::LayerPosition;
use std::mem;

#[derive(Debug)]
pub struct Layer<B> where B: Target {
	storage: PackedBlockStorage<LayerPosition>,
	palette: Palette<B>
}

impl<B> Layer<B> where B: Target {
	pub fn new(bits_per_entry: usize, default: B) -> Self {
		Layer {
			storage: PackedBlockStorage::new(bits_per_entry),
			palette: Palette::new(bits_per_entry, default)
		}
	}
	
	/// Increased the capacity of this chunk's storage by 1 bit, and returns the old storage for reuse purposes.
	pub fn reserve_bits(&mut self, bits: usize) -> PackedBlockStorage<LayerPosition> {
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
	
	pub fn get(&self, position: LayerPosition) -> PaletteAssociation<B> {
		self.storage.get(position, &self.palette)
	}
	
	// TODO: Methods to work with the palette: pruning, etc.
	
	pub fn palette_mut(&mut self) -> &mut Palette<B> {
		&mut self.palette
	}
	
	pub fn freeze_read_only(&self) -> (&PackedBlockStorage<LayerPosition>, &Palette<B>) {
		(&self.storage, &self.palette)
	}
	
	pub fn freeze_palette(&mut self) -> (&mut PackedBlockStorage<LayerPosition>, &Palette<B>) {
		(&mut self.storage, &self.palette)
	}
	
	/// Preforms the ensure_available, reverse_lookup, and set calls all in one.
	/// Prefer freezing the palette for larger scale block sets.
	pub fn set_immediate(&mut self, position: LayerPosition, target: &B) {
		self.ensure_available(target.clone());
		let association = self.palette.reverse_lookup(&target).unwrap();
		
		self.storage.set(position, &association, &mut NullRecorder);
	}
}