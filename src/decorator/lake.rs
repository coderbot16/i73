struct LakeSettings {
	horizontal: usize,
	vertical: usize,
	
	surface: i32,
	max_y: i32,
	max_h: i32,
	base_blobs: i32,
	add_blobs: i32
}

impl Default for LakeSettings {
	fn default() -> Self {
		LakeSettings {
			horizontal: 16,
			vertical: 8,
			surface: 4,
			max_y: 7,
			max_h: 15,
			base_blobs: 4,
			add_blobs: 3
		}
	}
}

struct LakeShape {
	horizontal: usize,
	vertical: usize,
	liquid: BitVec
}

impl LakeShape {
	fn new(settings: &LakeSettings) -> Self {
		LakeShape {
			horizontal: settings.horizontal,
			vertical: settings.vertical,
			liquid: BitVec::from_elem(settings.horizontal * settings.horizontal * settings.vertical, false)
		}
	}
	
	fn clear(&mut self) {
		self.liquid.clear()
	}
	
	fn set(&mut self, x: usize, y: usize, z: usize, bit: bool) {
		self.dliquid.set((x * self.horizontal + z) * self.vertical + y, bit);
	}
	
	fn fill(&mut self, settings: &LakeSettings, rng: &mut JavaRng) -> Self {
		let blobs = settings.base_blobs + rng.next_i32(settings.add_blobs + 1);
		
		let h = self.horizontal as f64;
		let v = self.vertical as f64;
		
		for _ in 0..blobs {
			let diameter = (
				rng.next_f64() * 6.0 + 3.0, 
				rng.next_f64() * 4.0 + 2.0, 
				rng.next_f64() * 6.0 + 3.0
			);
			
			let radius = (
				diameter.0 / 2.0,
				diameter.1 / 2.0,
				diameter.2 / 2.0
			);
			
			let center = (
				rng.next_f64() * (h - diameter.0 - 2.0) + 1.0 + radius.0, 
				rng.next_f64() * (v - diameter.1 - 4.0) + 2.0 + radius.1, 
				rng.next_f64() * (h - diameter.2 - 2.0) + 1.0 + radius.2
			);
			
			// TODO: Reduce size of possible bounding box.
			for x in 1..settings.max_h {
				for y in 1..settings.max_y {
					for z in 1..settings.max_h {
						let axis_distances = (
				 			(x as f64 - center.0) / radius.0,
				 			(y as f64 - center.1) / radius.1,
				 			(z as f64 - center.2) / radius.2,
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
	}
	
	
}