pub mod village;

#[derive(Serialize, Deserialize, Debug)]
struct Structure {
	                          id: String,
	#[serde(rename="ChunkX")] chunk_x: i32,
	#[serde(rename="ChunkZ")] chunk_z: i32,
	#[serde(rename="BB")]     bounding_box: BoundingBox
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BoundingBox(i32, i32, i32, i32, i32, i32);
impl BoundingBox {
	fn lower(&self) -> (i32, i32, i32) {
		(self.0, self.1, self.2)
	}
	
	fn upper(&self) -> (i32, i32, i32) {
		(self.3, self.4, self.5)
	}
}