use rng::JavaRng;
use bit_vec::BitVec;
use chunk::storage::Target;
use chunk::matcher::BlockMatcher;
use chunk::grouping::{Moore, Result};

pub struct LakeBlocks<B, L, S, R> where B: Target, L: BlockMatcher<B>, S: BlockMatcher<B>, R: BlockMatcher<B> {
	pub is_liquid:  L,
	pub is_solid:   S,
	pub replacable: R,
	pub liquid:     B,
	pub carve:      B,
	pub solidify:   Option<B>
}

impl<B, L, S, R> LakeBlocks<B, L, S, R> where B: Target, L: BlockMatcher<B>, S: BlockMatcher<B>, R: BlockMatcher<B> {
	pub fn check_border(&self, shape: &LakeShape, moore: &mut Moore<B>, lower: (i32, i32, i32)) -> Result<bool> {
		
		for x in 0..shape.horizontal {
			for z in 0..shape.horizontal {
				let (x_i32, z_i32) = (x as i32, z as i32);
				
				for y in 0..shape.surface {
					if shape.get_border(x, y, z) {
						let association = moore.get((lower.0 + x_i32, lower.1 + y as i32, lower.2 + z_i32))?;
						let block = association.target()?;
						
						if *block != self.liquid && !self.is_solid.matches(block) {
							return Ok(false)
						}
					}
				}
				
				for y in shape.surface..shape.vertical {
					if shape.get_border(x, y, z) {
						if self.is_liquid.matches(moore.get((lower.0 + x_i32, lower.1 + y as i32, lower.2 + z_i32))?.target()?) {
							return Ok(false);
						}
					}
				}
			}
		}
		
		Ok(true)
	}
	
	pub fn fill_and_carve(&self, shape: &LakeShape, moore: &mut Moore<B>, lower: (i32, i32, i32)) -> Result<()> {
		moore.ensure_available(self.liquid.clone());
		moore.ensure_available(self.carve.clone());
		
		let (mut blocks, palette) = moore.freeze_palettes();
		
		let liquid = palette.reverse_lookup(&self.liquid).unwrap();
		let carve = palette.reverse_lookup(&self.carve).unwrap();
		
		for x in 0..shape.horizontal {
			for z in 0..shape.horizontal {
				for y in 0..shape.surface {
					let position = (lower.0 + x as i32, lower.1 + y as i32, lower.2 + z as i32);
					
					if shape.get(x, y, z) {
						blocks.set(position, &liquid);
					}
				}
				
				for y in shape.surface..shape.vertical {
					let position = (lower.0 + x as i32, lower.1 + y as i32, lower.2 + z as i32);
					
					if shape.get(x, y, z) {
						blocks.set(position, &carve);
					}
				}
			}
		}
		
		Ok(())
	}
	
	// TODO: grow_grass, solidify_border
}

pub struct LakeSettings {
	pub horizontal: usize,
	pub vertical: usize,
	pub surface: usize,
	
	pub min_blobs: i32,
	pub add_blobs: i32
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

pub struct LakeBlobs<'r> {
	horizontal:      f64,
	vertical:        f64,
	remaining_blobs: i32,
	rng:             &'r mut JavaRng
}

impl<'r> LakeBlobs<'r> {
	pub fn new(rng: &'r mut JavaRng, settings: &LakeSettings) -> Self {
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

#[derive(Debug)]
pub struct Blob {
	pub center: (f64, f64, f64),
	pub radius: (f64, f64, f64)
}

pub struct LakeShape {
	/// Horizontal size of the lake.
	horizontal: usize,
	/// Vertical size of the lake.
	vertical: usize,
	surface: usize,
	/// Defines the volume of the lake. 
	liquid: BitVec,
	/// Defines the blocks bordering the volume of the lake.
	border: BitVec
}

impl LakeShape {
	pub fn new(settings: &LakeSettings) -> Self {
		LakeShape {
			horizontal: settings.horizontal,
			vertical: settings.vertical,
			surface: settings.surface,
			liquid: BitVec::from_elem(settings.horizontal * settings.horizontal * settings.vertical, false),
			border: BitVec::from_elem(settings.horizontal * settings.horizontal * settings.vertical, false)
		}
	}
	
	pub fn clear(&mut self) {
		self.liquid.clear()
	}
	
	pub fn set(&mut self, x: usize, y: usize, z: usize, bit: bool) {
		self.liquid.set((x * self.horizontal + z) * self.vertical + y, bit);
	}
	
	pub fn get(&self, x: usize, y: usize, z: usize) -> bool {
		self.liquid.get((x * self.horizontal + z) * self.vertical + y).unwrap()
	}
	
	pub fn set_border(&mut self, x: usize, y: usize, z: usize, bit: bool) {
		self.border.set((x * self.horizontal + z) * self.vertical + y, bit);
	}
	
	pub fn get_border(&self, x: usize, y: usize, z: usize) -> bool {
		self.border.get((x * self.horizontal + z) * self.vertical + y).unwrap()
	}
	
	pub fn fill(&mut self, blobs: LakeBlobs) {
		for blob in blobs {
			self.add_blob(blob);
		}
	}
	
	pub fn add_blob(&mut self, blob: Blob) {
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
					
					let preexisting = self.get(x, y, z);
					self.set(x, y, z, preexisting || distance_squared < 1.0);
		 		}
		 	}
		}
	}
	
	pub fn update_border(&mut self) {
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