use matcher::DeprecatedBlockMatcher;
use vocs::indexed::Target;
use vocs::view::QuadMut;
use vocs::position::{QuadPosition, Offset, dir};
use decorator::{Decorator, Result};
use java_rand::Random;

pub struct TreeDecorator<S, M, B> where S: DeprecatedBlockMatcher<B>, M: DeprecatedBlockMatcher<B>, B: Target {
	blocks: TreeBlocks<S, M, B>,
	settings: TreeSettings
}

impl<S, M, B> Decorator<B> for TreeDecorator<S, M, B> where S: DeprecatedBlockMatcher<B>, M: DeprecatedBlockMatcher<B>, B: Target {
	fn generate(&self, quad: &mut QuadMut<B>, rng: &mut Random, position: QuadPosition) -> Result {
		let tree = self.settings.tree(rng, position);
		
		if tree.leaves_max_y > 128 {
			return Ok(());
		}

		let below = match position.offset(dir::Down) {
			Some(below) => below,
			None => return Ok(())
		};

		if !self.blocks.soil.matches(quad.get(below)) {
			return Ok(());
		}
		
		// TODO: Check bounding box
		
		quad.set_immediate(below, &self.blocks.new_soil);
		
		quad.ensure_available(self.blocks.log.clone());
		quad.ensure_available(self.blocks.foliage.clone());
		
		let (mut blocks, palette) = quad.freeze_palette();
		
		let log = palette.reverse_lookup(&self.blocks.log).unwrap();
		let foliage = palette.reverse_lookup(&self.blocks.foliage).unwrap();
		
		for y in tree.leaves_min_y..tree.leaves_max_y {
			let radius = tree.foliage_radius(y) as i32;
			
			for z_offset in -radius..radius {
				for x_offset in -radius..radius {
					if z_offset.abs() != radius || x_offset.abs() != radius || rng.next_u32_bound(self.settings.foliage_corner_chance) != 0 && y < tree.trunk_top {

						let position = match position.offset((x_offset as i8, 0, z_offset as i8)) {
							Some(position) => position,
							None => continue
						};

						let position = QuadPosition::new(position.x(), y as u8, position.z());
						
						blocks.set(position, &foliage);
					}
				}
			}
		}
		
		for y in position.y()..(tree.trunk_top as u8) {
			let position = QuadPosition::new(position.x(), y, position.z());
			
			if self.blocks.replace.matches(blocks.get(position, &palette)) {
				blocks.set(position, &log);
			}
		}
		
		Ok(())
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

struct TreeBlocks<S, M, B> where S: DeprecatedBlockMatcher<B>, M: DeprecatedBlockMatcher<B>, B: Target {
	log:      B,
	foliage:  B,
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
			foliage:  18*16,
			replace:  replacable,
			soil:     is_soil,
			new_soil: 3*16
		}
	}
}

struct TreeSettings {
	min_trunk_height: u32,
	add_trunk_height: u32,
	foliage_layers_on_trunk: u32,
	foliage_layers_off_trunk: u32,
	foliage_slope: u32,
	foliage_radius_base: u32,
	foliage_corner_chance: u32
}

impl TreeSettings {
	fn tree(&self, rng: &mut Random, orgin: QuadPosition) -> Tree {
		let trunk_height = self.min_trunk_height + rng.next_u32_bound(self.add_trunk_height + 1);
		let trunk_top = (orgin.y() as u32) + trunk_height;
		
		Tree {
			orgin,
			full_height: trunk_height + self.foliage_layers_off_trunk,
			trunk_height,
			trunk_top,
			leaves_min_y: trunk_top - self.foliage_layers_on_trunk,
			leaves_max_y: trunk_top + self.foliage_layers_off_trunk,
			leaves_slope: self.foliage_slope,
			leaves_radius_base: self.foliage_radius_base
		}
	}
}

impl Default for TreeSettings {
	fn default() -> Self {
		TreeSettings {
			min_trunk_height: 4,
			add_trunk_height: 2,
			foliage_layers_on_trunk: 3,
			foliage_layers_off_trunk: 1,
			foliage_slope: 2,
			foliage_radius_base: 1,
			foliage_corner_chance: 2
		}
	}
}

struct Tree {
	orgin: QuadPosition,
	/// Trunk Height + number of foliage layers above the trunk
	full_height: u32,
	/// Height of the trunk. Can be considered the length of the line that defines the trunk.
	trunk_height: u32,
	/// Coordinates of the block above the last block of the trunk.
	trunk_top: u32,
	/// Minimum Y value for foliage layers (Inclusive).
	leaves_min_y: u32,
	/// Maximum Y value for foliage layers (Exclusive).
	leaves_max_y: u32,
	/// Slope value of the radius for each layer. Flattens or widens the tree.
	leaves_slope: u32,
	/// Base value for the radius.
	leaves_radius_base: u32
}

impl Tree {
	/// Radius of the foliage at a given location. 0 is just the trunk.
	fn foliage_radius(&self, y: u32) -> u32 {
		(self.leaves_radius_base + self.trunk_top - y) / self.leaves_slope
	}
	
	/// Radius of the bounding box for the foliage at a given level. 0 for just checking the trunk.
	fn bounding_radius(&self, y: u32) -> u32 {
		if y == (self.orgin.y() as u32) {
			0
		} else if y > self.trunk_top {
			2
		} else {
			1
		}
	}
}