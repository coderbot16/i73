use std::collections::HashMap;
use chunk::position::BlockPosition;

#[derive(Debug, Serialize, Deserialize)]
pub struct ChunkRoot {
	#[serde(rename="DataVersion")]
	pub version: i32,
	#[serde(rename="Level")]
	pub chunk: Chunk
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Chunk {
	#[serde(rename="xPos")]
	pub x: i32,
	#[serde(rename="zPos")]
	pub z: i32,
	#[serde(rename="LastUpdate")]
	pub last_update: i64,
	#[serde(rename="LightPopulated")]
	pub light_populated: bool,
	#[serde(rename="TerrainPopulated")]
	pub terrain_populated: bool,
	#[serde(rename="V")]
	pub v: i8,
	#[serde(rename="InhabitedTime")]
	pub inhabited_time: i64,
	#[serde(rename="Biomes")]
	pub biomes: Vec<i8>,
	#[serde(rename="HeightMap")]
	pub heightmap: Vec<i32>,
	#[serde(rename="Sections")]
	pub sections: Vec<Section>,
	#[serde(rename="Entities")]
	pub entities: Vec<HashMap<String, ()>>,
	#[serde(rename="TileEntities")]
	pub tile_entities: Vec<HashMap<String, ()>>,
	#[serde(rename="TileTicks")]
	pub tile_ticks: Vec<HashMap<String, ()>>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Section {
	#[serde(rename="Y")]
	pub y: i8,
	#[serde(rename="Blocks")]
	pub blocks: Vec<i8>,
	#[serde(rename="Add")]
	pub add: Option<NibbleVec>,
	#[serde(rename="Data")]
	pub data: NibbleVec,
	#[serde(rename="BlockLight")]
	pub block_light: NibbleVec,
	#[serde(rename="SkyLight")]
	pub sky_light: NibbleVec
}

impl Section {
	fn get_light(&self, at: BlockPosition) -> Light {
		Light((self.sky_light.get(at) << 4) | self.block_light.get(at))
	}
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NibbleVec(Vec<i8>);
impl NibbleVec {
	pub fn new() -> Self {
		NibbleVec(Vec::new())
	}
	
	pub fn with_capacity(nibbles: usize) -> Self {
		NibbleVec(Vec::with_capacity(nibbles / 2))
	}
	
	pub fn filled() -> Self {
		NibbleVec(vec![0; 2048])
	}
	
	fn get(&self, at: BlockPosition) -> i8 {
		let (index, shift) = nibble_index(at);
		(self.0[index]&(0xF << shift)) >> shift
	}
	
	pub fn set(&mut self, at: BlockPosition, value: i8) {
		let (index, shift) = nibble_index(at);
		let cleared = !(!self.0[index]) | (0xF << shift);
		self.0[index] = cleared | ((value&0xF) << shift);
	}
	
	/// Version of `NibbleVec::set` that doesn't clear the value at the position to 0 before preforming bitwise or.
	/// Use when you know that the value at that position is 0.
	pub fn set_uncleared(&mut self, at: BlockPosition, value: i8) {
		let (index, shift) = nibble_index(at);
		self.0[index] |= (value&0xF) << shift;
	}
}

// Add<<12 | Block<<4 | Data
struct AnvilId(u16);
// Sky<<4 | Block
struct Light(i8);

fn nibble_index(at: BlockPosition) -> (usize, i8) {
	let raw = at.chunk_yzx();
	((raw >> 1) as usize, (raw & 1) as i8 * 4)
}