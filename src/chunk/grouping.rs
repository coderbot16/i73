use chunk::storage::{Chunk, Palette, PackedBlockStorage, PaletteAssociation};
use chunk::position::BlockPosition;
use std::hash::Hash;
use totuple::{array_to_tuple_mut_16, array_to_tuple_16, array_to_tuple_mut_9, array_to_tuple_9};

pub struct Column<B>([Chunk<B>; 16]) where B: Eq + Hash + Clone;

impl<B> Column<B> where B: Eq + Hash + Clone {
	pub fn with_bits(bits_per_entry: usize) -> Self {
		Column([
			Chunk::new(bits_per_entry), Chunk::new(bits_per_entry), Chunk::new(bits_per_entry), Chunk::new(bits_per_entry),
			Chunk::new(bits_per_entry), Chunk::new(bits_per_entry), Chunk::new(bits_per_entry), Chunk::new(bits_per_entry),
			Chunk::new(bits_per_entry), Chunk::new(bits_per_entry), Chunk::new(bits_per_entry), Chunk::new(bits_per_entry),
			Chunk::new(bits_per_entry), Chunk::new(bits_per_entry), Chunk::new(bits_per_entry), Chunk::new(bits_per_entry)
		])
	}
	
	pub fn new(chunks: [Chunk<B>; 16]) -> Self {
		Column(chunks)
	}
	
	pub fn chunk(&self, index: usize) -> &Chunk<B> {
		&self.0[index]
	}
	
	pub fn chunk_mut(&mut self, index: usize) -> &mut Chunk<B> {
		&mut self.0[index]
	}
	
	pub fn freeze_palettes(&mut self) -> (ColumnBlocks, ColumnPalettes<B>) {
		let chunks = array_to_tuple_mut_16(&mut self.0);
		
		let frozen = (
			chunks. 0.freeze_palette(), chunks. 1.freeze_palette(), chunks. 2.freeze_palette(), chunks. 3.freeze_palette(),
			chunks. 4.freeze_palette(), chunks. 5.freeze_palette(), chunks. 6.freeze_palette(), chunks. 7.freeze_palette(),
			chunks. 8.freeze_palette(), chunks. 9.freeze_palette(), chunks.10.freeze_palette(), chunks.11.freeze_palette(),
			chunks.12.freeze_palette(), chunks.13.freeze_palette(), chunks.14.freeze_palette(), chunks.15.freeze_palette()
		);
		
		(
			ColumnBlocks ([
				(frozen. 0).0, (frozen. 1).0, (frozen. 2).0, (frozen. 3).0,
				(frozen. 4).0, (frozen. 5).0, (frozen. 6).0, (frozen. 7).0,
				(frozen. 8).0, (frozen. 9).0, (frozen.10).0, (frozen.11).0,
				(frozen.12).0, (frozen.13).0, (frozen.14).0, (frozen.15).0
			]),
			ColumnPalettes([
				(frozen. 0).1, (frozen. 1).1, (frozen. 2).1, (frozen. 3).1,
				(frozen. 4).1, (frozen. 5).1, (frozen. 6).1, (frozen. 7).1,
				(frozen. 8).1, (frozen. 9).1, (frozen.10).1, (frozen.11).1,
				(frozen.12).1, (frozen.13).1, (frozen.14).1, (frozen.15).1
			])
		)
	}
}

pub struct ColumnBlocks<'a>([&'a mut PackedBlockStorage; 16]);
impl<'a> ColumnBlocks<'a> {
	fn get<'p, B>(&self, at: BlockPosition, palettes: ColumnPalettes<'p, B>) -> PaletteAssociation<'p, B> where B: Eq + Hash + Clone {
		let chunk_y = at.chunk_y() as usize;
		
		self.0[chunk_y].get(at, palettes.0[chunk_y])
	}
	
	fn set<B>(&mut self, at: BlockPosition, association: ColumnAssociation<B>) where B: Eq + Hash + Clone {
		let chunk_y = at.chunk_y() as usize;
		
		self.0[chunk_y].set(at, &association.0[chunk_y])
	}
}

pub struct ColumnPalettes<'a, B>([&'a Palette<B>; 16]) where B: 'a + Eq + Hash + Clone;
impl<'a, B> ColumnPalettes<'a, B> where B: 'a + Eq + Hash + Clone {
	/// Gets an association that will reference back to the target. Note that several indices may point to the same target, this returns one of them.
	pub fn reverse_lookup(&self, target: &B) -> Result<ColumnAssociation<B>, usize> {
		let palettes = array_to_tuple_16(&self.0);
		
		Ok(ColumnAssociation ([
			palettes. 0.reverse_lookup(target).ok_or( 0usize)?,
			palettes. 1.reverse_lookup(target).ok_or( 1usize)?,
			palettes. 2.reverse_lookup(target).ok_or( 2usize)?,
			palettes. 3.reverse_lookup(target).ok_or( 3usize)?,
			palettes. 4.reverse_lookup(target).ok_or( 4usize)?,
			palettes. 5.reverse_lookup(target).ok_or( 5usize)?,
			palettes. 6.reverse_lookup(target).ok_or( 6usize)?,
			palettes. 7.reverse_lookup(target).ok_or( 7usize)?,
			palettes. 8.reverse_lookup(target).ok_or( 8usize)?,
			palettes. 9.reverse_lookup(target).ok_or( 9usize)?,
			palettes.10.reverse_lookup(target).ok_or(10usize)?,
			palettes.11.reverse_lookup(target).ok_or(11usize)?,
			palettes.12.reverse_lookup(target).ok_or(12usize)?,
			palettes.13.reverse_lookup(target).ok_or(13usize)?,
			palettes.14.reverse_lookup(target).ok_or(14usize)?,
			palettes.15.reverse_lookup(target).ok_or(15usize)?,
		]))
	}
}

pub struct ColumnAssociation<'a, B>([PaletteAssociation<'a, B>; 16]) where B: 'a + Eq + Hash + Clone;

pub struct Moore<B> where B: Eq + Hash + Clone {
	columns: [Column<B>; 9]
}

impl<B> Moore<B> where B: Eq + Hash + Clone {
	pub fn new(columns: [Column<B>; 9]) -> Self {
		Moore { columns }
	}
	
	pub fn column(&self, relative: (i8, i8)) -> &Column<B> {
		&self.columns[index_relative(relative)]
	}
	
	pub fn column_mut(&mut self, relative: (i8, i8)) -> &mut Column<B> {
		&mut self.columns[index_relative(relative)]
	}
	
	pub fn freeze_palettes(&mut self) -> (MooreBlocks, MoorePalettes<B>) {
		let columns = array_to_tuple_mut_9(&mut self.columns);
		
		let frozen = (
			columns. 0.freeze_palettes(), columns. 1.freeze_palettes(), columns. 2.freeze_palettes(), 
			columns. 3.freeze_palettes(), columns. 4.freeze_palettes(), columns. 5.freeze_palettes(), 
			columns. 6.freeze_palettes(), columns. 7.freeze_palettes(), columns. 8.freeze_palettes()
		);
		
		(
			MooreBlocks ([
				(frozen. 0).0, (frozen. 1).0, (frozen. 2).0, 
				(frozen. 3).0, (frozen. 4).0, (frozen. 5).0, 
				(frozen. 6).0, (frozen. 7).0, (frozen. 8).0, 
			]),
			MoorePalettes([
				(frozen. 0).1, (frozen. 1).1, (frozen. 2).1, 
				(frozen. 3).1, (frozen. 4).1, (frozen. 5).1, 
				(frozen. 6).1, (frozen. 7).1, (frozen. 8).1, 
			])
		)
	}
}

pub struct MooreBlocks<'a>([ColumnBlocks<'a>; 9]);
pub struct MoorePalettes<'a, B>([ColumnPalettes<'a, B>; 9]) where B: 'a + Eq + Hash + Clone;
impl<'a, B> MoorePalettes<'a, B> where B: 'a + Eq + Hash + Clone {
	/// Gets an association that will reference back to the target. Note that several indices may point to the same target, this returns one of them.
	pub fn reverse_lookup(&self, target: &B) -> Result<MooreAssociation<B>, (usize, usize)> {
		let palettes = array_to_tuple_9(&self.0);
		
		Ok(MooreAssociation ([
			palettes. 0.reverse_lookup(target).map_err(|e| (e, 0usize))?,
			palettes. 1.reverse_lookup(target).map_err(|e| (e, 1usize))?,
			palettes. 2.reverse_lookup(target).map_err(|e| (e, 2usize))?,
			palettes. 3.reverse_lookup(target).map_err(|e| (e, 3usize))?,
			palettes. 4.reverse_lookup(target).map_err(|e| (e, 4usize))?,
			palettes. 5.reverse_lookup(target).map_err(|e| (e, 5usize))?,
			palettes. 6.reverse_lookup(target).map_err(|e| (e, 6usize))?,
			palettes. 7.reverse_lookup(target).map_err(|e| (e, 7usize))?,
			palettes. 8.reverse_lookup(target).map_err(|e| (e, 8usize))?
		]))
	}
}

pub struct MooreAssociation<'a, B>([ColumnAssociation<'a, B>; 9]) where B: 'a + Eq + Hash + Clone;

fn index(actual: (u8, u8)) -> usize {
	(actual.0 * 3 + actual.1) as usize
}

fn index_relative(relative: (i8, i8)) -> usize {
	index(((relative.0 + 1) as u8, (relative.1 + 1) as u8))
}