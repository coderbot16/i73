use matcher::BlockMatcher;
use vocs::indexed::Target;
use chunk::grouping::{Moore, Result};
use decorator::Decorator;
use java_rand::Random;

pub struct TreeDecorator<S, M, B> where S: BlockMatcher<B>, M: BlockMatcher<B>, B: Target {
	blocks: TreeBlocks<S, M, B>,
	settings: TreeSettings
}

impl<S, M, B> Decorator<B> for TreeDecorator<S, M, B> where S: BlockMatcher<B>, M: BlockMatcher<B>, B: Target {
	fn generate(&self, moore: &mut Moore<B>, rng: &mut Random, position: (i32, i32, i32)) -> Result<bool> {
		let tree = self.settings.tree(rng, position);
		
		if position.1 < 1 || tree.leaves_max_y > 128 {
			return Ok(false);
		}
		
		if !self.blocks.soil.matches(moore.get((position.0, position.1 - 1, position.2))?.target()?) {
			return Ok(false);
		}
		
		// TODO: Check bounding box
		
		moore.set_immediate((position.0, position.1 - 1, position.2), &self.blocks.new_soil)?;
		
		moore.ensure_available(self.blocks.log.clone());
		moore.ensure_available(self.blocks.foilage.clone());
		
		let (mut blocks, palette) = moore.freeze_palette();
		
		let log = palette.reverse_lookup(&self.blocks.log).unwrap();
		let foilage = palette.reverse_lookup(&self.blocks.foilage).unwrap();
		
		for y in tree.leaves_min_y..tree.leaves_max_y {
			let radius = tree.foilage_radius(y);
			
			for z_offset in -radius..radius {
				for x_offset in -radius..radius {
					if z_offset.abs() != radius || x_offset.abs() != radius || rng.next_i32(self.settings.foilage_corner_chance) != 0 && y < tree.trunk_top {
						let at = (position.0 + x_offset, y, position.1 + z_offset);
						
						blocks.set(at, &foilage)?;
					}
				}
			}
		}
		
		for y in position.1..tree.trunk_top {
			let position = (position.0, y, position.2);
			
			if self.blocks.replace.matches(blocks.get(position, &palette)?.target()?) {
				blocks.set(position, &log)?;
			}
		}
		
		Ok(true)
	}
}

impl Default for TreeDecorator<fn(&u16) -> bool, fn(&u16) -> bool, u16> {
	fn default() -> Self {
		TreeDecorator {
			blocks: TreeBlocks::default(),
			settings: TreeSettings::default()
		}
	}
}

struct TreeBlocks<S, M, B> where S: BlockMatcher<B>, M: BlockMatcher<B>, B: Target {
	log:      B,
	foilage:  B,
	replace:  M,
	soil:     S,
	new_soil: B
}

fn replacable(block: &u16) -> bool {
	*block == 0*16 || *block == 18*16
}

fn is_soil(block: &u16) -> bool {
	*block == 2*16 || *block == 3*16
}

impl Default for TreeBlocks<fn(&u16) -> bool, fn(&u16) -> bool, u16> {
	fn default() -> Self {
		TreeBlocks {
			log:      17*16,
			foilage:  18*16,
			replace:  replacable,
			soil:     is_soil,
			new_soil: 3*16
		}
	}
}

struct TreeSettings {
	min_trunk_height: i32,
	add_trunk_height: i32,
	foilage_layers_on_trunk: i32,
	foilage_layers_off_trunk: i32,
	foilage_slope: i32,
	foilage_radius_base: i32,
	foilage_corner_chance: i32
}

impl TreeSettings {
	fn tree(&self, rng: &mut Random, orgin: (i32, i32, i32)) -> Tree {
		let trunk_height = self.min_trunk_height + rng.next_i32(self.add_trunk_height + 1);
		let trunk_top = orgin.1 + trunk_height;
		
		Tree {
			orgin,
			full_height: trunk_height + self.foilage_layers_off_trunk,
			trunk_height,
			trunk_top,
			leaves_min_y: orgin.1 + trunk_height - self.foilage_layers_on_trunk,
			leaves_max_y: trunk_top + self.foilage_layers_off_trunk,
			leaves_slope: self.foilage_slope,
			leaves_radius_base: self.foilage_radius_base
		}
	}
}

impl Default for TreeSettings {
	fn default() -> Self {
		TreeSettings {
			min_trunk_height: 4,
			add_trunk_height: 2,
			foilage_layers_on_trunk: 3,
			foilage_layers_off_trunk: 1,
			foilage_slope: 2,
			foilage_radius_base: 1,
			foilage_corner_chance: 2
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
	fn foilage_radius(&self, y: i32) -> i32 {
		(self.leaves_radius_base + self.trunk_top - y) / self.leaves_slope
	}
	
	/// Radius of the bounding box for the foilage at a given level. 0 for just checking the trunk.
	fn bounding_radius(&self, y: i32) -> i32 {
		if y == self.orgin.0 {
			0
		} else if y > self.trunk_top {
			2
		} else {
			1
		}
	}
}