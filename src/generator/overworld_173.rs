use rng::JavaRng;
use noise::octaves::PerlinOctaves;
use climate::ClimateSource;
use noise_field::height::{HeightSettings, HeightSource};
use noise_field::volume::{TriNoiseSettings, TriNoiseSource, FieldSettings, H_NOISE_SIZE, Y_NOISE_SIZE};
use generator::Pass;
use chunk::position::BlockPosition;
use chunk::storage::Target;
use chunk::grouping::{Column, Result, ColumnBlocks, ColumnPalettes, ColumnAssociation};
use chunk::matcher::{BlockMatcher, Is, IsNot};
use sample::Sample;
use nalgebra::{Vector2, Vector3};
use noise_field::height::lerp_to_layer;

pub struct Settings<R, I, B> where R: BlockMatcher<B>, I: BlockMatcher<B>, B: Target {
	shape_blocks: ShapeBlocks<B>,
	paint_blocks: PaintBlocks<R, I, B>,
	tri:          TriNoiseSettings,
	height:       HeightSettings,
	field:        FieldSettings,
	sea_coord:    u8,
	max_bedrock_height: Option<u8>
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
			max_bedrock_height: Some(5)
		}
	}
}

pub fn passes<R, I, B>(seed: i64, settings: Settings<R, I, B>) -> (ShapePass<B>, PaintPass<R, I, B>) where R: BlockMatcher<B>, I: BlockMatcher<B>, B: Target {
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
	let climate = ClimateSource::new(seed);
	
	(
		ShapePass { climate, blocks: settings.shape_blocks, tri, height, field, sea_coord: settings.sea_coord },
		PaintPass { blocks: settings.paint_blocks, sand, gravel, thickness, sea_coord: settings.sea_coord, max_bedrock_height: settings.max_bedrock_height }
	)
}

pub struct ShapeBlocks<B> where B: Target {
	solid: B,
	ocean: B,
	ice:   B,
	air:   B
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
	fn apply(&self, target: &mut Column<B>, chunk: (i32, i32)) -> Result<()> {
		let offset = Vector2::new(
			(chunk.0 as f64) * 4.0,
			(chunk.1 as f64) * 4.0
		);
		
		let block_offset = (
			(chunk.0 as f64) * 16.0,
			(chunk.1 as f64) * 16.0
		);
		
		let climate_chunk = self.climate.chunk(block_offset);
		
		let mut field = [[[0f64; H_NOISE_SIZE]; Y_NOISE_SIZE]; H_NOISE_SIZE];
	
		for x in 0..H_NOISE_SIZE {
			for z in 0..H_NOISE_SIZE {
				let layer = lerp_to_layer(Vector2::new(x, z));
				
				let climate = climate_chunk.get(layer.x, layer.y);
				let height = self.height.sample(offset + Vector2::new(x as f64, z as f64), climate);
				
				for y in 0..Y_NOISE_SIZE {
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
			let position = BlockPosition::from_yzx(i);
			let altitude = position.y();
			
			let block = if trilinear(&field, position) > 0.0 {
				&solid
			} else if altitude == self.sea_coord && climate_chunk.get(position.x() as usize, position.z() as usize).temperature() < 0.5 {
				&ice
			} else if altitude <= self.sea_coord {
				&ocean
			} else {
				&air
			};
			
			blocks.set(position, block);
		}
		
		Ok(())
	}
}

pub struct PaintBlocks<R, I, B> where R: BlockMatcher<B>, I: BlockMatcher<B>, B: Target {
	reset:     R,
	ignore:    I,
	air:       B,
	stone:     B,
	ocean:     B,
	gravel:    B,
	sand:      B,
	sandstone: B,
	top_todo:  B,
	fill_todo: B,
	bedrock:   B
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
			top_todo:   2 * 16, // Grass
			fill_todo:  3 * 16, // Dirt
			bedrock:    7 * 16
		}
	}
}

pub struct PaintAssociations<'a, B> where B: 'a + Target {
	air:       ColumnAssociation<'a, B>,
	stone:     ColumnAssociation<'a, B>,
	ocean:     ColumnAssociation<'a, B>,
	gravel:    ColumnAssociation<'a, B>,
	sand:      ColumnAssociation<'a, B>,
	sandstone: ColumnAssociation<'a, B>,
	top_todo:  ColumnAssociation<'a, B>,
	fill_todo: ColumnAssociation<'a, B>,
	bedrock:   ColumnAssociation<'a, B>
}

pub struct PaintPass<R, I, B> where R: BlockMatcher<B>, I: BlockMatcher<B>, B: Target {
	blocks:    PaintBlocks<R, I, B>,
	sand:      PerlinOctaves,
	gravel:    PerlinOctaves,
	thickness: PerlinOctaves,
	sea_coord: u8,
	max_bedrock_height: Option<u8>
}

impl<R, I, B> PaintPass<R, I, B> where R: BlockMatcher<B>, I: BlockMatcher<B>, B: Target {
	fn paint_stack(&self, rng: &mut JavaRng, blocks: &mut ColumnBlocks, palette: &ColumnPalettes<B>, associations: &PaintAssociations<B>, x: u8, z: u8, /*TODO: biome: Biome<B, I>,*/ sand: bool, gravel: bool, thickness: i32) {	
		let reset_remaining = match thickness {
			-1          => None,
			x if x <= 0 => Some(0),
			thickness   => Some(thickness as u32)
		};
		
		// TODO: Configuability
		let beach_min = self.sea_coord - 4;
		let beach_max = self.sea_coord + 1;
		
		let mut remaining = None;
		let mut followup_index: Option<usize> = None;
		let mut     top_block = &associations.top_todo;
		let mut current_block = &associations.fill_todo;
		
		for y in (0..128).map(|y| 127 - y) {
			let position = BlockPosition::new(x, y, z);
			
			if let Some(chance) = self.max_bedrock_height {
				if (y as i32) <= rng.next_i32(chance as i32) {
					blocks.set(position, &associations.bedrock);
					continue;
				}
			}
			
			let empty = if y <= self.sea_coord {&associations.ocean} else {&associations.air};
			
			let existing = blocks.get(position, &palette);
			let target = existing.target();
			
			match target {
				Ok(block) => if self.blocks.reset.matches(block) { 
					remaining = None; 
					continue 
				} else       if self.blocks.ignore.matches(block) { 
					continue 
				},
				Err(_) => continue
			}
			
			match remaining {
				Some(0) => (),
				Some(ref mut remaining) => {
					blocks.set(position, current_block);
					
					*remaining -= 1;
					if *remaining == 0 {
						// TODO: Followup surfaces (Sand => Sandstone, etc).
						// TODO: rng.next_i32(4);
					}
				},
				None => {
					if thickness <= 0 {
						top_block     = empty;
						current_block = &associations.stone;
					} else if y >= beach_min && y <= beach_max {
						if sand {
							top_block     = &associations.sand;
							current_block = &associations.sand;
						} else if gravel {
							top_block     = empty;
							current_block = &associations.gravel;
						} else {
							top_block     = &associations.top_todo;
							current_block = &associations.fill_todo;
						}
					}
			
					blocks.set(position, if y >= self.sea_coord {top_block} else {current_block});
			
					remaining = reset_remaining;
				}
			}
			
			
		}
	}
}

impl<R, I, B> Pass<B> for PaintPass<R, I, B> where R: BlockMatcher<B>, I: BlockMatcher<B>, B: Target {
	fn apply(&self, target: &mut Column<B>, chunk: (i32, i32)) -> Result<()> {
		let block = ((chunk.0 * 16) as f64, (chunk.1 * 16) as f64);
		let mut rng = JavaRng::new((chunk.0 as i64).wrapping_mul(341873128712).wrapping_add((chunk.1 as i64).wrapping_mul(132897987541)));
		
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
		target.ensure_available(self.blocks.top_todo.clone());
		target.ensure_available(self.blocks.fill_todo.clone());
		target.ensure_available(self.blocks.bedrock.clone());
		
		let (mut blocks, palette) = target.freeze_palettes();
		
		let associations = PaintAssociations {
			air:        palette.reverse_lookup(&self.blocks.air).unwrap(),
			stone:      palette.reverse_lookup(&self.blocks.stone).unwrap(),
			ocean:      palette.reverse_lookup(&self.blocks.ocean).unwrap(),
			gravel:     palette.reverse_lookup(&self.blocks.gravel).unwrap(),
			sand:       palette.reverse_lookup(&self.blocks.sand).unwrap(),
			sandstone:  palette.reverse_lookup(&self.blocks.sandstone).unwrap(),
			top_todo:   palette.reverse_lookup(&self.blocks.top_todo).unwrap(),
			fill_todo:  palette.reverse_lookup(&self.blocks.fill_todo).unwrap(),
			bedrock:    palette.reverse_lookup(&self.blocks.bedrock).unwrap()
		};
		
		for z in 0..16usize {
			for x in 0..16usize {
				let (sand_variation, gravel_variation, thickness_variation) = (rng.next_f64() * 0.2, rng.next_f64() * 0.2, rng.next_f64() * 0.25);

				let   sand    =       sand_vertical.generate_override(  vertical_offset + Vector3::new(x as f64, z as f64, 0.0), z) +   sand_variation > 0.0;
				let gravel    =         self.gravel.sample           (horizontal_offset + Vector2::new(x as f64, z as f64     )   ) + gravel_variation > 3.0;
				let thickness = (thickness_vertical.generate_override(  vertical_offset + Vector3::new(x as f64, z as f64, 0.0), z) / 3.0 + 3.0 + thickness_variation) as i32;
				
				self.paint_stack(&mut rng, &mut blocks, &palette, &associations, x as u8, z as u8, sand, gravel, thickness);
			}
		}
		
		Ok(())
	}
}

fn trilinear(array: &[[[f64; H_NOISE_SIZE]; Y_NOISE_SIZE]; H_NOISE_SIZE], position: BlockPosition) -> f64 {
	let inner = (
		((position.x() % 4) as f64) / 4.0,
		((position.y() % 8) as f64) / 8.0,
		((position.z() % 4) as f64) / 4.0
	);
	
	let indices = (
		(position.x() / 4) as usize,
		(position.y() / 8) as usize,
		(position.z() / 4) as usize
	);
	
	lerp(inner.2, 
		lerp(inner.0,
			lerp(inner.1,
				array[indices.0    ][indices.1    ][indices.2    ],
				array[indices.0    ][indices.1 + 1][indices.2    ],
			),
			lerp(inner.1,
				array[indices.0 + 1][indices.1    ][indices.2    ],
				array[indices.0 + 1][indices.1 + 1][indices.2    ],
			)
		),
		lerp(inner.0,
			lerp(inner.1,
				array[indices.0    ][indices.1    ][indices.2 + 1],
				array[indices.0    ][indices.1 + 1][indices.2 + 1],
			),
			lerp(inner.1,
				array[indices.0 + 1][indices.1    ][indices.2 + 1],
				array[indices.0 + 1][indices.1 + 1][indices.2 + 1],
			)
		)
	)
}

fn lerp(t: f64, a: f64, b: f64) -> f64 {
	a + t * (b - a)
}