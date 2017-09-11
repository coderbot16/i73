use byteorder::{ByteOrder, BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Write, Read, Result};

pub struct RegionHeader<'a>(&'a [u8; 8192]);
impl<'a> RegionHeader<'a> {
	pub fn new(data: &'a [u8; 8192]) -> Self {
		RegionHeader(data)
	}
	
	/// Gets the location of this chunk in the file.
	/// # Panics
	/// If X or Z is greater than or equal to 32, the function will panic.
	pub fn location(&self, x: u8, z: u8) -> Option<ChunkLocation> {
		if x >= 32 || z >= 32 {
			panic!("Chunk location out of bounds in region file: {}, {}", x, z)
		}
		
		let idx = ((x as usize) | ((z as usize)<<5)) * 4;
		ChunkLocation::new(BigEndian::read_u32(&self.0[idx..]))
	}
	
	/// Gets the timestamp this chunk was saved at.
	/// # Panics
	/// If X or Z is greater than or equal to 32, the function will panic.
	pub fn timestamp(&self, x: u8, z: u8) -> ChunkTimestamp {
		if x > 32 || z > 32 {
			panic!("Chunk location out of bounds in region file: {}, {}", x, z)
		}
		
		let idx = ((x as usize) | ((z as usize)<<5)) * 4 + 4096;
		ChunkTimestamp::new(BigEndian::read_u32(&self.0[idx..]))
	}
}

pub struct ChunkLocation(u32);
impl ChunkLocation {
	pub fn new(loc: u32) -> Option<Self> {
		if loc == 0 {
			None
		} else {
			Some(ChunkLocation(loc))
		}
	}
	
	/// Returns the contained raw value, which is gaurunteed to be non-zero.
	pub fn inner(&self) -> u32 {
		self.0
	}
	
	/// Returns the offset in pages (4096 bytes) of this chunk from the start of the file.
	pub fn offset(&self) -> u32 {
		self.0 >> 8
	}
	
	/// Returns the offset in bytes of this chunk from the start of the file.
	pub fn offset_bytes(&self) -> u64 {
		(self.offset() as u64) * 4096
	}

	/// Returns the size of the chunk in pages (4096 bytes).
	pub fn len(&self) -> u8 {
		(self.0 & 0xFF) as u8
	}
	
	/// Returns the size of the chunk in bytes.
	pub fn len_bytes(&self) -> u32 {
		(self.0 & 0xFF) * 4096
	}
}

// TODO: What is a chunk timestamp?
pub struct ChunkTimestamp(u32);
impl ChunkTimestamp {
	pub fn new(loc: u32) -> Self {
		ChunkTimestamp(loc)
	}
	
	pub fn inner(&self) -> u32 {
		self.0
	}
}

pub struct ChunkHeader {pub len: u32, pub compression: u8}

impl ChunkHeader {
	pub fn read<R: Read>(r: &mut R) -> Result<Self> {
		Ok(ChunkHeader {
			len: r.read_u32::<BigEndian>()?,
			compression: r.read_u8()?
		})
	}
	
	pub fn write<W: Write>(&self, w: &mut W) -> Result<()> {
		w.write_u32::<BigEndian>(self.len)?;
		w.write_u8(self.compression)
	}
}