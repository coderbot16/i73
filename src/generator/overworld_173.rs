use rng::JavaRng;
use noise::Permutations;
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
	tri:          TriNoiseSettings,
	height:       HeightSettings,
	field:        FieldSettings,
	sea_coord:    u8
}

impl Default for Settings<u16> {
	fn default() -> Self {
		Settings {
			shape_blocks: ShapeBlocks::default(),
			tri:          TriNoiseSettings::default(),
			height:       HeightSettings::default(),
			field:        FieldSettings::default(),
			sea_coord:    63
		}
	}
}

pub fn passes<B>(seed: i64, settings: Settings<B>) -> (ShapePass<B>, PaintPass) where B: Target {
	let mut rng = JavaRng::new(seed);
	
	let tri = TriNoiseSource::new(&mut rng, &settings.tri);
	
	// TODO: The PerlinOctaves implementation currently does not support noise on arbitrary Y coordinates.
	// Oddly, this "feature" is what causes the sharp walls in beach/biome surfaces.
	// It is a mystery why the feature exists in the first place.
	
	let beach = [
		Permutations::new(&mut rng),
		Permutations::new(&mut rng),
		Permutations::new(&mut rng),
		Permutations::new(&mut rng)
	];
	
	let thickness = [
		Permutations::new(&mut rng),
		Permutations::new(&mut rng),
		Permutations::new(&mut rng),
		Permutations::new(&mut rng)
	];
	
	let height = HeightSource::new(&mut rng, &settings.height);
	let field = settings.field;
	let climate = ClimateSource::new(seed);
	
	(
		ShapePass { climate, blocks: settings.shape_blocks, tri, height, field, sea_coord: settings.sea_coord },
		PaintPass { beach, thickness }
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

pub struct PaintPass {
	beach:     [Permutations; 4], // TODO
	thickness: [Permutations; 4]  // TODO
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