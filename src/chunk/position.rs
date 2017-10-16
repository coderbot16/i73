use chunk::storage::PackedIndex;
use std::fmt::{Debug, Formatter, Result};

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct BlockPosition(u16);

impl BlockPosition {
	/// Creates a new BlockPosition from the X, Y, and Z components.
	/// ### Out of bounds behavior
	/// If the arguments are out of bounds, then they are truncated.
	pub fn new(x: u8, y: u8, z: u8) -> Self {
		BlockPosition(
			((y as u16) << 8) | 
			(((z&0xF) as u16) << 4) | 
			((x&0xF) as u16)
		)
	}
	
	/// Creates a new BlockPosition from a YZX index.
	/// ### Out of bounds behavior
	/// If the index is out of bounds, it is truncated.
	pub fn from_yzx(yzx: u16) -> Self {
		BlockPosition(yzx)
	}
	
	/// Creates a new BlockPosition from a XYZ index.
	/// ### Out of bounds behavior
	/// If the index is out of bounds, it is truncated.
	pub fn from_xyz(xyz: u16) -> Self {
		let xyz = xyz & 0xFFF; // Truncate the value if too large
		// X YZ - Start
		// YZ X - End
		BlockPosition(((xyz & 0xF00) >> 8) | ((xyz & 0x0FF) << 4))
	}
	
	/// Returns the X component.
	pub fn x(&self) -> u8 {
		(self.0 & 0x00F) as u8
	}
	
	/// Returns the Z component.
	pub fn z(&self) -> u8 {
		((self.0 & 0x0F0) >> 4) as u8
	}
	
	/// Returns the Y component.
	pub fn y(&self) -> u8 {
		(self.0 >> 8) as u8
	}
	
	/// Returns the Y component >> 4, the chunk Y.
	pub fn chunk_y(&self) -> u8 {
		(self.0 >> 12) as u8
	}
	
	/// Returns the Y and Z components, represented as `(Y<<4) | Z`.
	pub fn yz(&self) -> u16 {
		self.0 >> 4
	}
	
	/// Returns the index represented as `(Z<<4) | X`.
	pub fn zx(&self) -> u8 {
		(self.0 & 255) as u8
	}
	
	/// Returns the index represented as `(Y<<8) | (Z<<4) | X`.
	pub fn yzx(&self) -> u16 {
		self.0
	}
	
	/// Returns the index represented as `(Y<<8) | (Z<<4) | X` modulo 4096, for in-chunk indices.
	pub fn chunk_yzx(&self) -> u16 {
		self.0 & 4095
	}
	
	/// Returns the index represented as `(X<<8) | (Y<<4) | Z`.
	pub fn xyz(&self) -> u16 {
		((self.x() as u16) << 8) | self.yz()
	}
	
	/// Returns the chunk_yzx index into a nibble array. Returns in the form (index, shift).
	pub fn chunk_nibble_yzx(&self) -> (usize, i8) {
		let raw = self.chunk_yzx();
		((raw >> 1) as usize, (raw & 1) as i8 * 4)
	}
	
	pub fn minus_x(&self) -> Option<BlockPosition> {
		if self.x() != 0 {
			Some(BlockPosition(self.0 - 0x0001))
		} else {
			None
		}
	}
	
	pub fn plus_x(&self) -> Option<BlockPosition> {
		if self.x() != 15 {
			Some(BlockPosition(self.0 + 0x0001))
		} else {
			None
		}
	}
	
	pub fn minus_z(&self) -> Option<BlockPosition> {
		if self.z() != 0 {
			Some(BlockPosition(self.0 - 0x0010))
		} else {
			None
		}
	}
	
	pub fn plus_z(&self) -> Option<BlockPosition> {
		if self.z() != 15 {
			Some(BlockPosition(self.0 + 0x0010))
		} else {
			None
		}
	}
	
	pub fn minus_y(&self) -> Option<BlockPosition> {
		if self.y() > 0 {
			Some(BlockPosition(self.0 - 0x0100))
		} else {
			None
		}
	}
	
	pub fn plus_y(&self) -> Option<BlockPosition> {
		if self.y() <= 15 {
			Some(BlockPosition(self.0 + 0x0100))
		} else {
			None
		}
	}
}

impl PackedIndex for BlockPosition {
	fn entries() -> usize {
		4096
	}
	
	fn from_index(index: usize) -> Self {
		BlockPosition::from_yzx(index as u16)
	}
	
	fn to_index(&self) -> usize {
		self.chunk_yzx() as usize
	}
}

impl Debug for BlockPosition {
	fn fmt(&self, f: &mut Formatter) -> Result {
		write!(f, "BlockPosition {{ x: {}, y: {}, z: {}, yzx: {} }}", self.x(), self.y(), self.z(), self.yzx())
	}
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct LayerPosition(u8);

impl LayerPosition {
	/// Creates a new LayerPosition from the X and Z components.
	/// ### Out of bounds behavior
	/// If the arguments are out of bounds, then they are truncated.
	pub fn new(x: u8, z: u8) -> Self {
		LayerPosition(((z&0xF) << 4) | (x&0xF))
	}
	
	/// Creates a new LayerPosition from a ZX index.
	/// ### Out of bounds behavior
	/// If the index is out of bounds, it is truncated.
	pub fn from_zx(zx: u8) -> Self {
		LayerPosition(zx)
	}
	
	/// Returns the X component.
	pub fn x(&self) -> u8 {
		self.0 & 0x0F
	}
	
	/// Returns the Z component.
	pub fn z(&self) -> u8 {
		self.0 >> 4
	}
	
	/// Returns the index represented as `(Z<<4) | X`.
	pub fn zx(&self) -> u8 {
		self.0
	}
}

impl PackedIndex for LayerPosition {
	fn entries() -> usize {
		256
	}
	
	fn from_index(index: usize) -> Self {
		LayerPosition::from_zx(index as u8)
	}
	
	fn to_index(&self) -> usize {
		self.zx() as usize
	}
}