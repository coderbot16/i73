use rng::NotchRng;
use image_ops::filter::Source;
use image_ops::Image;

pub struct Continents {
	pub chance: i32,
	pub rng: NotchRng
}

impl Source for Continents {
	type Out = bool;
	
	fn fill(&self, position: (i64, i64), image: &mut Image<bool>) {
		for z in 0..image.z_size() {
			for x in 0..image.x_size() {
				let mut rng = self.rng.clone();
				
				rng.init_at(position.0 + (x as i64), position.1 + (z as i64));
				
				image.set(x, z, rng.next_i32(self.chance) == 0);
			}
		}
		
		// Make sure the area near spawn has a continent.
		
		if position.0 > -(image.x_size() as i64) && position.0 <= 0 && 
		   position.1 > -(image.z_size() as i64) && position.1 <= 0 {
			image.set((-position.0) as usize, (-position.1) as usize, true);
		}
	}
}