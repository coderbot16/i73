use java_rand::Random;
use std::cmp::min;

const TAU: f64 = 2.0 * 3.14159;

/// A foilage cluster. "Balloon" oaks in Minecraft are simply a large tree generating a single foilage cluster at the top of the very short trunk.
#[derive(Debug)]
pub struct Foilage {
	/// Location of the leaf cluster, and the endpoint of the branch line. The Y is at the bottom of the cluster.
	cluster: (i32, i32, i32),
	/// Y coordinate of the block above the top of this foilage cluster.
	foilage_top_y: i32,
	/// Y coordinate of the start of the branch line. The X and Z coordinate are always equal to the orgin of the tree.
	branch_y: i32
}

#[derive(Debug)]
pub struct LargeTreeSettings {
	/// Makes the branches shorter or longer than the default.
	branch_scale: f64,
	/// For every 1 block the branch is long, this multiplier determines how many blocks it will go down on the trunk.
	branch_slope: f64,
	/// Default height of the leaves of the foilage clusters, from top to bottom. 
	/// When added to the Y of the cluster, represents the coordinate of the top layer of the leaf cluster.
	foilage_height: i32,
	/// Factor in determining the amount of foilage clusters generated on each Y level of the big tree.
	foilage_density: f64,
	/// Added to the foilage_per_y value before conversion to i32.
	base_foilage_per_y: f64, 
	/// How tall the trunk is in comparison to the total height. Should be 0.0 to 1.0.
	trunk_height_scale: f64,
	/// Minimum height of the tree.
	min_height: i32,
	/// Maximum height that can be added to the minimum. Max height of the tree = min_height + add_height.
	add_height: i32
}

impl Default for LargeTreeSettings {
	fn default() -> Self {
		LargeTreeSettings {
			branch_scale: 1.0,
			branch_slope: 0.381,
			foilage_height: 4,
			foilage_density: 1.0,
			base_foilage_per_y: 1.382,
			trunk_height_scale: 0.618,
			min_height: 5,
			add_height: 11
		}
	}
}

impl LargeTreeSettings {
	pub fn tree(&self, orgin: (i32, i32, i32), rng: &mut Random, preset_height: Option<i32>, max_height: i32) -> LargeTree {
		let height = min(preset_height.unwrap_or_else(|| self.min_height + rng.next_i32(self.add_height + 1)), max_height);
		let height_f32 = height as f32;
		let height_f64 = height as f64;
		
		let spread_center    = height_f32 / 2.0;
		let spread_center_sq = spread_center.powi(2);
		
		let trunk_height = min((height_f64 * self.trunk_height_scale) as i32, height - 1);
		let trunk_top    = orgin.1 + trunk_height;
		
		let foilage_per_y = ((self.foilage_density * height_f64 / 13.0).powi(2) + self.base_foilage_per_y).max(1.0) as i32;
		let foilage_max_y = orgin.1 + height - self.foilage_height;
		let foilage_min_y = orgin.1 + (height_f32 * 0.3).ceil() as i32 - 1;
		
		LargeTree { 
			orgin, 
			spread_center, 
			spread_center_sq, 
			height, 
			trunk_top, 
			foilage_per_y, 
			foilage_max_y, 
			foilage_min_y, 
			branch_scale: self.branch_scale,
			branch_slope: self.branch_slope,
			foilage_height: self.foilage_height
		}
	}
}

#[derive(Debug)]
pub struct LargeTree {
	/// Coordinate pointing to the bottom log of the trunk.
	pub orgin:            (i32, i32, i32),
	/// The place where most of the foilage should be centered around. At this point, the foilage should spread out the most. 
	/// If outside the range of the foilage range, then it will instead spread out the foilage the most at the bottom/top.
	pub spread_center:    f32,
	/// A squared version of the spread_center.
	pub spread_center_sq: f32,
	/// Maximum length from bottom to top of the tree.
	pub height:           i32,
	/// Y coordinate of the uppermost block of the trunk.
	pub trunk_top:        i32,
	/// Foilage per Y layer. At least 1.
	pub foilage_per_y:    i32,
	/// Maximum Y value for foilage layers to spawn (Inclusive). 
	/// A single foilage cluster is found at this value, centered on the trunk.
	/// For the purposes of iterating through the generated foilage layers, this can be considered Exclusive.
	pub foilage_max_y:    i32,
	/// Minimum Y value for foilage layers to spawn (Inclusive)
	pub foilage_min_y:    i32,
	/// Makes the branches shorter or longer than the default.
	pub branch_scale:     f64,
	/// For every 1 block the branch is long, this multiplier determines how many blocks it will go down on the trunk.
	pub branch_slope:     f64,
	/// Default height of the leaves of the foilage clusters, from top to bottom.
	pub foilage_height:   i32
}

impl LargeTree {
	/// Computes the spread at a given Y value. This is computed using the spread_center and spread_center_sq.
	pub fn spread(&self, y: i32) -> f64 {
		let distance_from_center = self.spread_center - (y - self.orgin.1 + 1) as f32;
		((self.spread_center_sq - distance_from_center.powi(2)).sqrt() * 0.5) as f64
	}
	
	// TODO: Replace this with an iterator implementation?
	/// Gets the foilage at a given Y level. The caller is responsible for ordering the calls, managing the Y value, and creating the random number generator.
	pub fn foilage(&self, y: i32, spread: f64, rng: &mut Random) -> Foilage {
		let branch_factor = self.branch_scale * spread * (rng.next_f32() as f64 + 0.328);
		let angle = (rng.next_f32() as f64) * TAU;
		
		let cluster = (
			(branch_factor * angle.sin() + (self.orgin.0 as f64) + 0.5).floor() as i32,
			y,
			(branch_factor * angle.cos() + (self.orgin.2 as f64) + 0.5).floor() as i32
		);
		
		let foilage_top_y = y + self.foilage_height;
		
		let trunk_distance = (
			(self.orgin.0 - cluster.0) as f64,
			(self.orgin.2 - cluster.2) as f64
		);
		
		let branch_length = (trunk_distance.0 * trunk_distance.0 + trunk_distance.1 * trunk_distance.1).sqrt();
		
		// Determine how low to place the branch start Y, controlled by branch_slope. Longer branches have lower starts on the trunk.
		let slope = branch_length * self.branch_slope;
		
		// Make sure the starting Y value for the branch is not above the trunk.
		// Interestingly, it does not check whether the branch starts below the trunk.
		let branch_y = ((y as f64) - slope).min(self.trunk_top as f64) as i32;
		
		// TODO: CheckLine from Cluster to TopY, and from BranchY to Cluster.
		Foilage { cluster, foilage_top_y, branch_y }
	}
}