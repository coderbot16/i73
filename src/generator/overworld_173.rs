use rng::JavaRng;
use noise::octaves::PerlinOctaves;
use biome::climate::{ClimateSettings, ClimateSource};
use biome::source::BiomeSource;
use biome::{Lookup, Surface};
use noise_field::height::{HeightSettings, HeightSource};
use noise_field::volume::{TriNoiseSettings, TriNoiseSource, FieldSettings, trilinear128};
use generator::Pass;
use vocs::position::{ColumnPosition, LayerPosition, GlobalColumnPosition};
use vocs::indexed::Target;
use vocs::world::view::{ColumnMut, ColumnBlocks, ColumnPalettes, ColumnAssociation};
use matcher::{BlockMatcher, Is, IsNot};
use sample::Sample;
use nalgebra::{Vector2, Vector3};
use noise_field::height::lerp_to_layer;

pub struct Settings<R, I, B> where R: BlockMatcher<B>, I: BlockMatcher<B>, B: Target {  
	pub shape_blocks: ShapeBlocks<B>,
	pub paint_blocks: PaintBlocks<R, I, B>,
	pub tri:          TriNoiseSettings,
	pub height:       HeightSettings,
	pub field:        FieldSettings,
	pub sea_coord:    u8,
	pub beach:        Option<(u8, u8)>,
	pub max_bedrock_height: Option<u8>,
	pub climate:      ClimateSettings
}

impl Default for Settings<Is<u16>, IsNot<u16>, u16> {
	fn default() -> Self {
		Settings {
			shape_blocks: ShapeBlocks::default(),
			paint_blocks: PaintBlocks::default(),
			tri:          TriNoiseSettings::default(),
			height:       HeightSettings::default(),
			field:        FieldSettings::default(),
			sea_coord:    63,
			beach:        Some((59, 65)),
			max_bedrock_height: Some(5),
			climate:      ClimateSettings::default()
		}
	}
}

pub fn passes<R, I, B>(seed: i64, settings: Settings<R, I, B>, biome_lookup: Lookup<B>) -> (ShapePass<B>, PaintPass<R, I, B>) where R: BlockMatcher<B>, I: BlockMatcher<B>, B: Target {
	let mut rng = JavaRng::new(seed);
	
	let tri = TriNoiseSource::new(&mut rng, &settings.tri);
	
	// TODO: The PerlinOctaves implementation currently does not support noise on arbitrary Y coordinates.
	// Oddly, this "feature" is what causes the sharp walls in beach/biome surfaces.
	// It is a mystery why the feature exists in the first place.
	
	let sand      = PerlinOctaves::new(&mut JavaRng { seed: rng.seed }, 4, Vector3::new(1.0 / 32.0, 1.0 / 32.0,        1.0)); // Vertical,   Z =   0.0
	let gravel    = PerlinOctaves::new(&mut rng,                        4, Vector3::new(1.0 / 32.0,        1.0, 1.0 / 32.0)); // Horizontal
	let thickness = PerlinOctaves::new(&mut rng,                        4, Vector3::new(1.0 / 16.0, 1.0 / 16.0, 1.0 / 16.0)); // Vertical,   Z =   0.0
	
	let height  = HeightSource::new(&mut rng, &settings.height);
	let field   = settings.field;
	let climate = ClimateSource::new(seed, settings.climate);
	
	(
		ShapePass { 
			climate, 
			blocks: settings.shape_blocks, 
			tri, 
			height, 
			field, 
			sea_coord: settings.sea_coord 
		},
		PaintPass { 
			biomes: BiomeSource::new(ClimateSource::new(seed, settings.climate), biome_lookup), 
			blocks: settings.paint_blocks, 
			sand, 
			gravel, 
			thickness, 
			sea_coord: settings.sea_coord, 
			beach: settings.beach,
			max_bedrock_height: settings.max_bedrock_height 
		}
	)
}

pub struct ShapeBlocks<B> where B: Target {
	pub solid: B,
	pub ocean: B,
	pub ice:   B,
	pub air:   B
}

impl Default for ShapeBlocks<u16> {
	fn default() -> Self {
		ShapeBlocks {
			solid:  1 * 16,
			ocean:  9 * 16,
			ice:   79 * 16,
			air:    0 * 16
		}
	}
}

pub struct ShapePass<B> where B: Target {
	climate: ClimateSource,
	blocks:  ShapeBlocks<B>,
	tri:     TriNoiseSource,
	height:  HeightSource,
	field:   FieldSettings,
	sea_coord: u8
}

impl<B> Pass<B> for ShapePass<B> where B: Target {
	fn apply(&self, target: &mut ColumnMut<B>, chunk: GlobalColumnPosition) {
		let offset = Vector2::new(
			(chunk.x() as f64) * 4.0,
			(chunk.z() as f64) * 4.0
		);
		
		let block_offset = (
			(chunk.x() as f64) * 16.0,
			(chunk.z() as f64) * 16.0
		);
		
		let climate_chunk = self.climate.chunk(block_offset);
		
		let mut field = [[[0f64; 5]; 17]; 5];
	
		for x in 0..5 {
			for z in 0..5 {
				let layer = lerp_to_layer(Vector2::new(x, z));
				
				let climate = climate_chunk.get(layer.x, layer.y);
				let height = self.height.sample(offset + Vector2::new(x as f64, z as f64), climate);
				
				for y in 0..17 {
					let tri = self.tri.sample(Vector3::new(offset.x + x as f64, y as f64, offset.y + z as f64), y);
					
					field[x][y][z] = self.field.compute_noise_value(y as f64, height, tri);
				}
			}
		}
		
		target.ensure_available(self.blocks.air.clone());
		target.ensure_available(self.blocks.solid.clone());
		target.ensure_available(self.blocks.ocean.clone());
		target.ensure_available(self.blocks.ice.clone());
		
		let (mut blocks, palette) = target.freeze_palettes();
		
		let air   = palette.reverse_lookup(&self.blocks.air).unwrap();
		let solid = palette.reverse_lookup(&self.blocks.solid).unwrap();
		let ocean = palette.reverse_lookup(&self.blocks.ocean).unwrap();
		let ice   = palette.reverse_lookup(&self.blocks.ice).unwrap();
		
		for i in 0..32768 {
			let position = ColumnPosition::from_yzx(i);
			let altitude = position.y();
			
			let block = if trilinear128(&field, position) > 0.0 {
				&solid
			} else if altitude == self.sea_coord && climate_chunk.get(position.x() as usize, position.z() as usize).freezing() {
				&ice
			} else if altitude <= self.sea_coord {
				&ocean
			} else {
				&air
			};
			
			blocks.set(position, block);
		}
	}
}

pub struct PaintBlocks<R, I, B> where R: BlockMatcher<B>, I: BlockMatcher<B>, B: Target {
	pub reset:     R,
	pub ignore:    I,
	pub air:       B,
	pub stone:     B,
	pub ocean:     B,
	pub gravel:    B,
	pub sand:      B,
	pub sandstone: B,
	pub bedrock:   B
}

impl Default for PaintBlocks<Is<u16>, IsNot<u16>, u16> {
	fn default() -> Self {
		PaintBlocks {
			reset:     Is   (0 * 16),
			ignore:    IsNot(1 * 16),
			air:        0 * 16,
			stone:      1 * 16,
			ocean:      9 * 16,
			gravel:    13 * 16,
			sand:      12 * 16,
			sandstone: 24 * 16,
			bedrock:    7 * 16
		}
	}
}

pub struct PaintAssociations {
	air:       ColumnAssociation,
	stone:     ColumnAssociation,
	ocean:     ColumnAssociation,
	bedrock:   ColumnAssociation
}

struct SurfaceAssociations {
	pub top:  ColumnAssociation,
	pub fill: ColumnAssociation,
	pub chain: Vec<FollowupAssociation>
}

impl SurfaceAssociations {
	fn lookup<B>(surface: &Surface<B>, palette: &ColumnPalettes<B>) -> Self where B: Target {
		let mut chain = Vec::new();
		
		for followup in &surface.chain {
			chain.push(
				FollowupAssociation {
					block:     palette.reverse_lookup(&followup.block).unwrap(),
					max_depth: followup.max_depth
				} 
			)
		}
		
		SurfaceAssociations {
			top:   palette.reverse_lookup(&surface.top).unwrap(),
			fill:  palette.reverse_lookup(&surface.fill).unwrap(),
			chain
		}
	}
}

struct FollowupAssociation {
	pub block:     ColumnAssociation,
	pub max_depth: u32
}

pub struct PaintPass<R, I, B> where R: BlockMatcher<B>, I: BlockMatcher<B>, B: Target {
	biomes:    BiomeSource<B>,
	blocks:    PaintBlocks<R, I, B>,
	sand:      PerlinOctaves,
	gravel:    PerlinOctaves,
	thickness: PerlinOctaves,
	sea_coord: u8,
	beach:     Option<(u8, u8)>,
	max_bedrock_height: Option<u8>
}

impl<R, I, B> PaintPass<R, I, B> where R: BlockMatcher<B>, I: BlockMatcher<B>, B: Target {
	fn paint_stack(&self, rng: &mut JavaRng, blocks: &mut ColumnBlocks, palette: &ColumnPalettes<B>, associations: &PaintAssociations, x: u8, z: u8, surface: &SurfaceAssociations, beach: &SurfaceAssociations, basin: &SurfaceAssociations, thickness: i32) {
		let reset_remaining = match thickness {
			-1          => None,
			x if x <= 0 => Some(0),
			thickness   => Some(thickness as u32)
		};
		
		let mut remaining = None;
		let mut followup_index: Option<usize> = None;
		
		let mut current_surface = if thickness <= 0 {basin} else {surface};
		
		for y in (0..128).rev() {
			let position = ColumnPosition::new(x, y, z);
			
			if let Some(chance) = self.max_bedrock_height {
				if (y as i32) <= rng.next_i32(chance as i32) {
					blocks.set(position, &associations.bedrock);
					continue;
				}
			}
			
			let existing = blocks.get(position, &palette);
			
			match existing {
				Some(block) => if self.blocks.reset.matches(block) {
					remaining = None; 
					continue 
				} else        if self.blocks.ignore.matches(block) {
					continue 
				},
				None => continue
			}
			
			match remaining {
				Some(0) => (),
				Some(ref mut remaining) => {
					let block = match followup_index {
						Some(index) => &current_surface.chain[index].block,
						None =>        &current_surface.fill
					};
					
					blocks.set(position, block);
					
					*remaining -= 1;
					if *remaining == 0 {
						// TODO: Don't increment the index if it is already out of bounds.
						let new_index = followup_index.map(|index| index + 1).unwrap_or(0);
						
						if new_index < current_surface.chain.len() {
							*remaining = rng.next_i32((current_surface.chain[new_index].max_depth as i32) + 1) as u32
						}
						
						followup_index = Some(new_index);
					}
				},
				None => {
					if thickness <= 0 {
						current_surface = basin;
					} else if let Some(beach_range) = self.beach {
						if y >= beach_range.0 && y <= beach_range.1 {
							current_surface = beach;
						}
					}
			
					blocks.set(position, if y >= self.sea_coord {&current_surface.top} else {&current_surface.fill});
				
					if y <= self.sea_coord && blocks.get(position, palette) == Some(&self.blocks.air) {
						blocks.set(position, &associations.ocean);
					}
			
					remaining = reset_remaining;
					followup_index = None;
				}
			}
			
			
		}
	}
}

impl<R, I, B> Pass<B> for PaintPass<R, I, B> where R: BlockMatcher<B>, I: BlockMatcher<B>, B: Target {
	fn apply(&self, target: &mut ColumnMut<B>, chunk: GlobalColumnPosition) {
		let block = ((chunk.x() * 16) as f64, (chunk.z() * 16) as f64);
		let mut rng = JavaRng::new((chunk.x() as i64).wrapping_mul(341873128712).wrapping_add((chunk.z() as i64).wrapping_mul(132897987541)));
		
		let biome_layer = self.biomes.layer(chunk);
		let (biomes, biome_palette) = biome_layer.freeze();
		
		let      sand_vertical = self.     sand.vertical_ref(block.1, 16);
		let thickness_vertical = self.thickness.vertical_ref(block.1, 16);
		
		let   vertical_offset = Vector3::new(block.0 as f64, block.1 as f64, 0.0);
		let horizontal_offset = Vector2::new(block.0 as f64, block.1 as f64);
		
		target.ensure_available(self.blocks.air.clone());
		target.ensure_available(self.blocks.stone.clone());
		target.ensure_available(self.blocks.ocean.clone());
		target.ensure_available(self.blocks.gravel.clone());
		target.ensure_available(self.blocks.sand.clone());
		target.ensure_available(self.blocks.sandstone.clone());
		target.ensure_available(self.blocks.bedrock.clone());
		
		for surface in biome_palette.entries().iter().filter_map(Option::as_ref).map(|biome| &biome.surface) {
			target.ensure_available(surface.top.clone());
			target.ensure_available(surface.fill.clone());
				
			for followup in &surface.chain {
				target.ensure_available(followup.block.clone());
			}
		}
		
		let (mut blocks, palette) = target.freeze_palettes();
		
		let mut surfaces = Vec::new();
		
		for entry in biome_palette.entries() {
			surfaces.push(
				entry.as_ref().map(|biome| SurfaceAssociations::lookup(&biome.surface, &palette))
			);
		}
		
		let associations = PaintAssociations {
			air:        palette.reverse_lookup(&self.blocks.air).unwrap(),
			stone:      palette.reverse_lookup(&self.blocks.stone).unwrap(),
			ocean:      palette.reverse_lookup(&self.blocks.ocean).unwrap(),
			bedrock:    palette.reverse_lookup(&self.blocks.bedrock).unwrap()
		};
		
		let gravel_beach = SurfaceAssociations {
			top:   palette.reverse_lookup(&self.blocks.air).unwrap(),
			fill:  palette.reverse_lookup(&self.blocks.gravel).unwrap(),
			chain: vec![]
		};
		
		let sand_beach   = SurfaceAssociations {
			top:   palette.reverse_lookup(&self.blocks.sand).unwrap(),
			fill:  palette.reverse_lookup(&self.blocks.sand).unwrap(),
			chain: vec![
				FollowupAssociation {
					block:     palette.reverse_lookup(&self.blocks.sandstone).unwrap(),
					max_depth: 3
				}
			]
		};
		
		let basin        = SurfaceAssociations {
			top:   palette.reverse_lookup(&self.blocks.air).unwrap(),
			fill:  palette.reverse_lookup(&self.blocks.stone).unwrap(),
			chain: vec![]
		};
		
		for z in 0..16 {
			for x in 0..16 {
				let position = LayerPosition::new(x, z);
				
				// TODO: BeachSelector
				
				let (sand_variation, gravel_variation, thickness_variation) = (rng.next_f64() * 0.2, rng.next_f64() * 0.2, rng.next_f64() * 0.25);

				let   sand    =       sand_vertical.generate_override(  vertical_offset + Vector3::new(x as f64, z as f64, 0.0), z as usize) +   sand_variation > 0.0;
				let gravel    =         self.gravel.sample           (horizontal_offset + Vector2::new(x as f64, z as f64     )            ) + gravel_variation > 3.0;
				let thickness = (thickness_vertical.generate_override(  vertical_offset + Vector3::new(x as f64, z as f64, 0.0), z as usize) / 3.0 + 3.0 + thickness_variation) as i32;

				let surface   = surfaces[biomes.get(position) as usize].as_ref().unwrap();
				
				let beach = if sand {
					&sand_beach
				} else if gravel {
					&gravel_beach
				} else {
					surface
				};
				
				self.paint_stack(&mut rng, &mut blocks, &palette, &associations, x, z, surface, beach, &basin, thickness);
			}
		}
	}
}