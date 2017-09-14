use rng::JavaRng;
use bit_vec::BitVec;

struct LakeSettings {
	horizontal: usize,
	vertical: usize,
	
	surface: i32,
	min_blobs: i32,
	add_blobs: i32
}

impl Default for LakeSettings {
	fn default() -> Self {
		LakeSettings {
			horizontal: 16,
			vertical:   8,
			surface:    4,
			min_blobs:  4,
			add_blobs:  3
		}
	}
}

struct LakeBlobs<'r> {
	horizontal:      f64,
	vertical:        f64,
	remaining_blobs: i32,
	rng:             &'r mut JavaRng
}

impl<'r> LakeBlobs<'r> {
	fn new(rng: &'r mut JavaRng, settings: &LakeSettings) -> Self {
		let remaining_blobs = settings.min_blobs + rng.next_i32(settings.add_blobs + 1);
		
		LakeBlobs { 
			horizontal: settings.horizontal as f64,
			vertical: settings.vertical as f64,
			remaining_blobs,
			rng
		}
	}
}

impl<'r> Iterator for LakeBlobs<'r> {
	type Item = Blob;
	
	fn next(&mut self) -> Option<Self::Item> {
		if self.remaining_blobs <= 0 {
			return None;
		}
		
		self.remaining_blobs -= 1;
		
		let diameter = (
			self.rng.next_f64() * 6.0 + 3.0, 
			self.rng.next_f64() * 4.0 + 2.0, 
			self.rng.next_f64() * 6.0 + 3.0
		);
			
		let radius = (
			diameter.0 / 2.0,
			diameter.1 / 2.0,
			diameter.2 / 2.0
		);
		
		let center = (
			self.rng.next_f64() * (self.horizontal - diameter.0 - 2.0) + 1.0 + radius.0, 
			self.rng.next_f64() * (self.vertical   - diameter.1 - 4.0) + 2.0 + radius.1, 
			self.rng.next_f64() * (self.horizontal - diameter.2 - 2.0) + 1.0 + radius.2
		);
		
		Some(Blob { radius, center })
	}
}

struct Blob {
	center: (f64, f64, f64),
	radius: (f64, f64, f64)
}

struct LakeShape {
	/// Horizontal size of the lake.
	horizontal: usize,
	/// Vertical size of the lake.
	vertical: usize,
	/// Defines the volume of the lake. 
	liquid: BitVec,
	/// Defines the blocks bordering the volume of the lake.
	border: BitVec
}

impl LakeShape {
	fn new(settings: &LakeSettings) -> Self {
		LakeShape {
			horizontal: settings.horizontal,
			vertical: settings.vertical,
			liquid: BitVec::from_elem(settings.horizontal * settings.horizontal * settings.vertical, false),
			border: BitVec::from_elem(settings.horizontal * settings.horizontal * settings.vertical, false)
		}
	}
	
	fn clear(&mut self) {
		self.liquid.clear()
	}
	
	fn set(&mut self, x: usize, y: usize, z: usize, bit: bool) {
		self.liquid.set((x * self.horizontal + z) * self.vertical + y, bit);
	}
	
	fn get(&self, x: usize, y: usize, z: usize) -> bool {
		self.liquid.get((x * self.horizontal + z) * self.vertical + y).unwrap()
	}
	
	fn set_border(&mut self, x: usize, y: usize, z: usize, bit: bool) {
		self.border.set((x * self.horizontal + z) * self.vertical + y, bit);
	}
	
	fn get_border(&self, x: usize, y: usize, z: usize) -> bool {
		self.border.get((x * self.horizontal + z) * self.vertical + y).unwrap()
	}
	
	fn fill(&mut self, blobs: LakeBlobs) {
		for blob in blobs {
			self.add_blob(blob);
		}
	}
	
	fn add_blob(&mut self, blob: Blob) {
		// TODO: Reduce size of possible bounding box.
		for x in 1..(self.horizontal - 1) {
			for y in 1..(self.vertical - 1) {
				for z in 1..(self.horizontal - 1) {
					let axis_distances = (
			 			(x as f64 - blob.center.0) / blob.radius.0,
			 			(y as f64 - blob.center.1) / blob.radius.1,
			 			(z as f64 - blob.center.2) / blob.radius.2,
					);
				
		 			let distance_squared = 
						axis_distances.0 * axis_distances.0 + 
						axis_distances.1 * axis_distances.1 + 
						axis_distances.2 * axis_distances.2;
					
					self.set(x, y, z, distance_squared < 1.0);
		 		}
		 	}
		}
	}
	
	fn update_border(&mut self) {
		self.border.clear();
		
		let y_max = self.vertical - 1;
		let h_max = self.horizontal - 1;
		
		// Main volume
		for x in 1..h_max {
			for y in 1..y_max {
				for z in 1..h_max {
					let is_border = !self.get(x, y, z) && (
						self.get(x + 1, y,     z    ) ||
						self.get(x - 1, y,     z    ) ||
						self.get(x,     y + 1, z    ) ||
						self.get(x,     y - 1, z    ) ||
						self.get(x,     y,     z + 1) ||
						self.get(x,     y,     z - 1)
					);
					
					self.set_border(x, y, z, is_border);
				}
			}
		}
		
		// Top and bottom face
		for x in 1..h_max {
			for z in 1..h_max {
				let bottom = self.get(x, 1,         z);
				let top    = self.get(x, y_max - 1, z);
				
				self.set_border(x, 0,     z, bottom);
				self.set_border(x, y_max, z, top   );
			}
		}
		
		// Z=0 / Z=Max faces
		for x in 1..h_max {
			for y in 1..y_max {
				let min = self.get(x, y, 1        );
				let max = self.get(x, y, h_max - 1);
				
				self.set_border(x, y, 0,     min);
				self.set_border(x, y, h_max, max);
			}
		}
		
		// X=0 / X=Max faces
		for z in 1..h_max {
			for y in 1..y_max {
				let min = self.get(1,         y, z);
				let max = self.get(h_max - 1, y, z);
				
				self.set_border(0,     y, z, min);
				self.set_border(h_max, y, z, max);
			}
		}
		
		// Skip the edge/corner cases (literally) as they cannot possibly fufill any of the criteria.
	}
}