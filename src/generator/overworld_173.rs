use rng::JavaRng;
use noise::octaves::PerlinOctaves;
use climate::ClimateSource;
use noise_field::height::{HeightSettings, HeightSource};
use noise_field::volume::{TriNoiseSettings, TriNoiseSource, FieldSettings, H_NOISE_SIZE, Y_NOISE_SIZE};
use generator::Pass;
use chunk::position::BlockPosition;
use chunk::storage::Target;
use chunk::grouping::{Column, Result};
use sample::Sample;
use nalgebra::{Vector2, Vector3};
use noise_field::height::lerp_to_layer;

pub struct Settings<B> where B: Target {
	shape_blocks: ShapeBlocks<B>,
	paint_blocks: PaintBlocks<B>,
	tri:          TriNoiseSettings,
	height:       HeightSettings,
	field:        FieldSettings,
	sea_coord:    u8
}

impl Default for Settings<u16> {
	fn default() -> Self {
		Settings {
			shape_blocks: ShapeBlocks::default(),
			paint_blocks: PaintBlocks::default(),
			tri:          TriNoiseSettings::default(),
			height:       HeightSettings::default(),
			field:        FieldSettings::default(),
			sea_coord:    63
		}
	}
}

pub fn passes<B>(seed: i64, settings: Settings<B>) -> (ShapePass<B>, PaintPass<B>) where B: Target {
	let mut rng = JavaRng::new(seed);
	
	let tri = TriNoiseSource::new(&mut rng, &settings.tri);
	
	// TODO: The PerlinOctaves implementation currently does not support noise on arbitrary Y coordinates.
	// Oddly, this "feature" is what causes the sharp walls in beach/biome surfaces.
	// It is a mystery why the feature exists in the first place.
	
	let sand      = PerlinOctaves::new(&mut JavaRng::new(rng.seed), 4, Vector3::new(1.0 / 32.0, 1.0 / 32.0,        1.0)); // Vertical,   Z =   0.0
	let gravel    = PerlinOctaves::new(&mut rng,                    4, Vector3::new(1.0 / 32.0,        1.0, 1.0 / 32.0)); // Horizontal, Y = 109.0134
	let thickness = PerlinOctaves::new(&mut rng,                    4, Vector3::new(1.0 / 16.0, 1.0 / 16.0, 1.0 / 16.0)); // Vertical,   Z =   0.0
	
	let height  = HeightSource::new(&mut rng, &settings.height);
	let field   = settings.field;
	let climate = ClimateSource::new(seed);
	
	(
		ShapePass { climate, blocks: settings.shape_blocks, tri, height, field, sea_coord: settings.sea_coord },
		PaintPass { blocks: settings.paint_blocks, sand, gravel, thickness }
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
			
			if trilinear(&field, position) > 0.0 {
				blocks.set(position, &solid);
			} else if altitude == self.sea_coord && climate_chunk.get(position.x() as usize, position.z() as usize).temperature() < 0.5 {
				blocks.set(position, &ice);
			} else if altitude <= self.sea_coord {
				blocks.set(position, &ocean);
			} else {
				blocks.set(position, &air);
			}
		}
		
		Ok(())
	}
}

pub struct PaintBlocks<B> where B: Target {
	air:       B,
	stone:     B,
	ocean:     B,
	gravel:    B,
	sand:      B,
	sandstone: B,
	top_todo:  B,
	fill_todo: B
}

impl Default for PaintBlocks<u16> {
	fn default() -> Self {
		PaintBlocks {
			air:        0 * 16,
			stone:      1 * 16,
			ocean:      9 * 16,
			gravel:    13 * 16,
			sand:      12 * 16,
			sandstone: 24 * 16,
			top_todo:   2 * 16, // Grass
			fill_todo:  3 * 16  // Dirt
		}
	}
}

pub struct PaintPass<B> where B: Target {
	blocks:    PaintBlocks<B>,
	sand:      PerlinOctaves,
	gravel:    PerlinOctaves,
	thickness: PerlinOctaves
}

impl<B> PaintPass<B> where B: Target {
	fn paint_stack(&self, /*_target: &mut Column<B>,*/ x: u8, z: u8, /*TODO: biome: BiomeDef<B, I>,*/ sand: bool, gravel: bool, thickness: i32) {
		// TODO: Target -> Blocks + Associations
	}
}

impl<B> Pass<B> for PaintPass<B> where B: Target {
	fn apply(&self, target: &mut Column<B>, chunk: (i32, i32)) -> Result<()> {
		let block = ((chunk.0 * 16) as f64, (chunk.1 * 16) as f64);
		let mut rng = JavaRng::new((chunk.0 as i64).wrapping_mul(341873128712).wrapping_add((chunk.1 as i64).wrapping_mul(132897987541)));
		
		let      sand_vertical = self.     sand.vertical_ref(block.1, 16);
		let thickness_vertical = self.thickness.vertical_ref(block.1, 16);
		
		let   vertical_offset = Vector3::new(block.0 as f64, block.1 as f64,            0.0);
		let horizontal_offset = Vector3::new(block.0 as f64,            0.0, block.1 as f64);
		
		target.ensure_available(self.blocks.air.clone());
		target.ensure_available(self.blocks.stone.clone());
		target.ensure_available(self.blocks.ocean.clone());
		target.ensure_available(self.blocks.gravel.clone());
		target.ensure_available(self.blocks.sand.clone());
		target.ensure_available(self.blocks.sandstone.clone());
		target.ensure_available(self.blocks.top_todo.clone());
		target.ensure_available(self.blocks.fill_todo.clone());
		
		let (mut blocks, palette) = target.freeze_palettes();
		
		let air       = palette.reverse_lookup(&self.blocks.air).unwrap();
		let stone     = palette.reverse_lookup(&self.blocks.stone).unwrap();
		let ocean     = palette.reverse_lookup(&self.blocks.ocean).unwrap();
		let gravel    = palette.reverse_lookup(&self.blocks.gravel).unwrap();
		let sand      = palette.reverse_lookup(&self.blocks.sand).unwrap();
		let sandstone = palette.reverse_lookup(&self.blocks.sandstone).unwrap();
		let top_todo  = palette.reverse_lookup(&self.blocks.top_todo).unwrap();
		let fill_todo = palette.reverse_lookup(&self.blocks.fill_todo).unwrap();
		
		for z in 0..16usize {
			for x in 0..16usize {
				let (sand_variation, gravel_variation, thickness_variation) = (rng.next_f64() * 0.2, rng.next_f64() * 0.2, rng.next_f64() * 0.25);

				let   sand    =       sand_vertical.generate_override(  vertical_offset + Vector3::new(x as f64, z as f64,      0.0), z) +   sand_variation > 0.0;
				let gravel    =         self.gravel.generate         (horizontal_offset + Vector3::new(x as f64, 109.0134, z as f64)   ) + gravel_variation > 3.0;
				let thickness = (thickness_vertical.generate_override(  vertical_offset + Vector3::new(x as f64, z as f64,      0.0), z) / 3.0 + 3.0 + thickness_variation) as i32;
				
				self.paint_stack(/*target,*/ x as u8, z as u8, sand, gravel, thickness);
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