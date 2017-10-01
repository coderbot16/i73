use byteorder::{ByteOrder, BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Write, Read, Result, Seek, SeekFrom, Error, ErrorKind};
use chunk::anvil::ChunkRoot;
use deflate::Compression;
use deflate::write::ZlibEncoder;
use nbt_serde::encode;

pub struct RegionWriter<O> where O: Write + Seek {
	header: Box<[u8; 8192]>,
	out: O,
	start: u64
}

impl<O> RegionWriter<O> where O: Write + Seek {
	pub fn start(mut out: O) -> Result<Self> {
		let start = out.seek(SeekFrom::Current(0))?;
		out.write_all(&[0; 8192])?;
		
		Ok(RegionWriter { header: Box::new([0; 8192]), out, start })
	}
	
	pub fn chunk(&mut self, x: u8, z: u8, chunk: &ChunkRoot) -> Result<u64> {
		let start = self.out.seek(SeekFrom::Current(0))?;
		// Write a fake placeholder header.
		ChunkHeader { len: 1, compression: 2 }.write(&mut self.out)?;
		
		{
			let mut encoder = ZlibEncoder::new(&mut self.out, Compression::Default);
			encode::to_writer(&mut encoder, &chunk, None).map_err(|e| Error::new(ErrorKind::Other, e.to_string()))?;
			encoder.finish()?;
		}
		
		let end = self.out.seek(SeekFrom::Current(0))?;
		
		let extents = end-start;
		let true_length = extents-4;
		assert!(true_length < (u32::max_value() as u64));
		
		// Fixup the placeholder header.
		self.out.seek(SeekFrom::Start(start))?;
		ChunkHeader { len: true_length as u32, compression: 2 }.write(&mut self.out)?;
		
		let location = ChunkLocation::from_parts(
			(start / 4096) as u32,
			(extents / 4096 + if extents % 4096 != 0 {1} else {0}) as u8
		);
		
		RegionHeaderMut::new(&mut self.header).location(x, z, location);
		
		self.out.seek(SeekFrom::Current(location.len_bytes() as i64 - 5)).map(|x| x - start)
	}
	
	pub fn finish(mut self) -> Result<()> {
		self.out.seek(SeekFrom::Start(self.start))?;
		self.out.write_all(&self.header[..])
	}
}

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

pub struct RegionHeaderMut<'a>(&'a mut [u8; 8192]);
impl<'a> RegionHeaderMut<'a> {
	pub fn new(data: &'a mut [u8; 8192]) -> Self {
		RegionHeaderMut(data)
	}
	
	/// Sets the location of this chunk in the file.
	/// # Panics
	/// If X or Z is greater than or equal to 32, the function will panic.
	pub fn location(&mut self, x: u8, z: u8, location: ChunkLocation) {
		if x >= 32 || z >= 32 {
			panic!("Chunk location out of bounds in region file: {}, {}", x, z)
		}
		
		let idx = ((x as usize) | ((z as usize)<<5)) * 4;
		BigEndian::write_u32(&mut self.0[idx..], location.0);
	}
	
	/// Sets the timestamp this chunk was saved at.
	/// # Panics
	/// If X or Z is greater than or equal to 32, the function will panic.
	pub fn timestamp(&mut self, x: u8, z: u8, timestamp: ChunkTimestamp) {
		if x > 32 || z > 32 {
			panic!("Chunk location out of bounds in region file: {}, {}", x, z)
		}
		
		let idx = ((x as usize) | ((z as usize)<<5)) * 4 + 4096;
		BigEndian::write_u32(&mut self.0[idx..], timestamp.0);
	}
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct ChunkLocation(u32);
impl ChunkLocation {
	pub fn from_parts(offset: u32, len: u8) -> Self {
		ChunkLocation((offset << 8) | (len as u32))
	}
	
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
	
	pub fn end(&self) -> u32 {
		self.offset() + (self.len() as u32)
	}
	
	pub fn end_bytes(&self) -> u32 {
		self.end() * 4096
	}
}

// TODO: What is a chunk timestamp?
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct ChunkTimestamp(u32);
impl ChunkTimestamp {
	pub fn new(loc: u32) -> Self {
		ChunkTimestamp(loc)
	}
	
	pub fn inner(&self) -> u32 {
		self.0
	}
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
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