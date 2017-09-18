struct TreeSettings {
	min_trunk_height: i32,
	add_trunk_height: i32,
	foilage_layers_on_trunk: i32,
	foilage_layers_off_trunk: i32,
	foilage_slope: i32,
	foilage_radius_base: i32
}

impl Default for TreeSettings {
	fn default() -> Self {
		TreeSettings {
			min_trunk_height: 4,
			add_trunk_height: 2,
			foilage_layers_on_trunk: 3,
			foilage_layers_off_trunk: 1,
			foilage_slope: 2,
			foilage_radius_base: 1
		}
	}
}

struct Tree {
	orgin: (i32, i32, i32),
	/// Trunk Height + number of foilage layers above the trunk
	full_height: i32,
	/// Height of the trunk. Can be considered the length of the line that defines the trunk.
	trunk_height: i32,
	/// Coordinates of the block above the last block of the trunk.
	trunk_top: i32,
	/// Minimum Y value for foilage layers (Inclusive).
	leaves_min_y: i32,
	/// Maximum Y value for foilage layers (Exclusive).
	leaves_max_y: i32,
	/// Slope value of the radius for each layer. Flattens or widens the tree.
	leaves_slope: i32,
	/// Base value for the radius.
	leaves_radius_base: i32
}

impl Tree {
	/// Radius of the foilage at a given location. 0 is just the trunk.
	fn foilage_radius(&self, y: i32) {
		self.leaves_radius_base - (y - trunk_top) / self.leaves_slope
	}
	
	/// Radius of the bounding box for the foilage at a given level. 0 for just checking the trunk.
	fn bounding_radius(&self, y: i32) {
		if y == self.orgin.0 {
			0
		} else if y > self.trunk_top {
			2
		} else {
			1
		}
	}
}