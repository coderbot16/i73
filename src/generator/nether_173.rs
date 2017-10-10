use chunk::storage::Target;
use chunk::position::BlockPosition;
use chunk::grouping::{Column, Result};
use generator::Pass;
use noise_field::volume::{self, TriNoiseSource, TriNoiseSettings, trilinear};
use nalgebra::{Vector2, Vector3};
use rng::JavaRng;

const NOTCH_PI_F64: f64 = 3.1415926535897931;

pub fn default_tri_settings() -> TriNoiseSettings {
	TriNoiseSettings {
		 main_out_scale:  20.0,
		upper_out_scale: 512.0,
		lower_out_scale: 512.0,
		lower_scale:     Vector3::new(684.412,        2053.236,        684.412       ),
		upper_scale:     Vector3::new(684.412,        2053.236,        684.412       ),
		 main_scale:     Vector3::new(684.412 / 80.0, 2053.236 / 60.0, 684.412 / 80.0),
		y_size:          33
	}
}

pub fn passes<B>(seed: i64, tri_settings: &TriNoiseSettings, blocks: ShapeBlocks<B>, sea_coord: u8) -> ShapePass<B> where B: Target {
	let mut rng = JavaRng::new(seed);
	
	let tri = TriNoiseSource::new(&mut rng, tri_settings);
	
	ShapePass {
		blocks,
		tri,
		reduction: generate_reduction_table(17),
		sea_coord
	}
}

pub struct ShapeBlocks<B> where B: Target {
	pub solid: B,
	pub air:   B,
	pub ocean: B
}

impl Default for ShapeBlocks<u16> {
	fn default() -> Self {
		ShapeBlocks {
			solid: 87 * 16,
			air:    0 * 16,
			ocean: 11 * 16
		}
	}
}

pub struct ShapePass<B> where B: Target {
	blocks:    ShapeBlocks<B>,
	tri:       TriNoiseSource,
	reduction: Vec<f64>,
	sea_coord: u8
}

impl<B> Pass<B> for ShapePass<B> where B: Target {
	fn apply(&self, target: &mut Column<B>, chunk: (i32, i32)) -> Result<()> {
		let offset = Vector2::new(
			(chunk.0 as f64) * 4.0,
			(chunk.1 as f64) * 4.0
		);
		
		let mut field = [[[0f64; 5]; 17]; 5];
	
		for x in 0..5 {
			for z in 0..5 {
				for y in 0..17 {
					let mut value = self.tri.sample(Vector3::new(offset.x + x as f64, y as f64, offset.y + z as f64), y);
					
					value -= self.reduction[y];
					value = volume::reduce_upper(value, y as f64, 4.0, 10.0, 17.0);
					
					field[x][y][z] = value;
				}
			}
		}
		
		target.ensure_available(self.blocks.air.clone());
		target.ensure_available(self.blocks.solid.clone());
		target.ensure_available(self.blocks.ocean.clone());
		
		let (mut blocks, palette) = target.freeze_palettes();
		
		let air   = palette.reverse_lookup(&self.blocks.air).unwrap();
		let solid = palette.reverse_lookup(&self.blocks.solid).unwrap();
		let ocean = palette.reverse_lookup(&self.blocks.ocean).unwrap();
		
		for i in 0..32768 {
			let position = BlockPosition::from_yzx(i);
			let altitude = position.y();
			
			let block = if trilinear(&field, position) > 0.0 {
				&solid
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

pub fn generate_reduction_table(y_size: usize) -> Vec<f64> {
	let mut data = Vec::with_capacity(y_size);
	let y_size_f64 = y_size as f64;
	
	for index in 0..y_size {
		let index_f64 = index as f64;
		
		let mut value = ((index_f64 * NOTCH_PI_F64 * 6.0) / y_size_f64).cos() * 2.0;
		
		value = volume::reduce_cubic(value, y_size_f64 - 1.0 - index_f64);
		value = volume::reduce_cubic(value, index_f64);
		
		data.push(value);
	}
	
	data
}