use std::collections::HashMap;

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
	/*#[serde(rename="Entities")]
	entities: Vec<HashMap<String, Value>>,
	#[serde(rename="TileEntities")]
	tile_entities: Vec<HashMap<String, Value>>,
	#[serde(rename="TileTicks")]
	tile_ticks: Vec<HashMap<String, Value>>*/
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
	fn set_block(&mut self, at: SectionCoords, id: AnvilId) {
		
	}
	
	fn get_block(&self, at: SectionCoords) -> AnvilId {
		match self.add {
			Some(ref add) => unimplemented!(),
			None => unimplemented!()
		}
	}
	
	fn get_light(&self, at: SectionCoords) -> Light {
		Light((self.sky_light.get(at) << 4) | self.block_light.get(at))
	}
}

#[derive(Debug, Copy, Clone)]
struct SectionCoords(u16);
impl SectionCoords {
	fn new(x: u8, y: u8, z: u8) -> Self {
		SectionCoords(((y as u16) << 8) | ((z as u16) << 4) | (x as u16))
	}
	
	fn nibble(&self) -> (usize, i8) {
		((self.0 >> 1) as usize, (self.0 & 1) as i8 * 4)
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NibbleVec(Vec<i8>);
impl NibbleVec {
	pub fn new() -> Self {
		NibbleVec(Vec::new())
	}
	
	pub fn with_capacity(nibbles: usize) -> Self {
		NibbleVec(Vec::with_capacity(nibbles / 2))
	}
	
	fn get(&self, at: SectionCoords) -> i8 {
		let (index, shift) = at.nibble();
		(self.0[index]&(0xF << shift)) >> shift
	}
}

// Add<<12 | Block<<4 | Data
struct AnvilId(u16);
// Sky<<4 | Block
struct Light(i8);