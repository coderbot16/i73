#![feature(exclusive_range_pattern)]

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate nbt_serde;
extern crate byteorder;
extern crate deflate;
extern crate bit_vec;

mod noise;
mod rng;
mod biome;
mod block;
mod sample;
mod climate;
mod surface;
mod noise_field;
mod decorator;
mod trig;
mod structure;
mod generator;
mod distribution;
mod chunk;

use rng::JavaRng;
use noise::{simplex, octaves, perlin, Permutations};
use sample::Sample;
use climate::ClimateSource;
use nalgebra::{Vector2, Vector3};
use noise_field::height::{lerp_to_layer, Height, HeightSettings, HeightSource};
use std::fs::File;
use nbt_serde::encode;

extern crate nalgebra;

fn main() {
	/*use decorator::large_tree::{LargeTreeSettings, LargeTree};
	let settings = LargeTreeSettings::default();
	
	for i in 0..1 {
		let mut rng = JavaRng::new(100 + i);
		let shape = settings.tree((0, 0, 0), &mut rng, None, 20);
		
		println!("{:?}", shape);
		
		let mut y = shape.foilage_max_y - 1;
		while y >= shape.foilage_min_y {
			let spread = shape.spread(y);
			
			println!("y: {}, spread: {}", y, spread);
			
			for _ in 0..shape.foilage_per_y {
				println!("{:?}", shape.foilage(y, spread, &mut rng));
			}
			
			y -= 1;
		}
	}*/
	
	use chunk::anvil::{ChunkRoot, Chunk, Section, NibbleVec};
	use chunk::region::RegionWriter;
	
	let root = ChunkRoot {
		version: 0,
		chunk: Chunk {
			x: 0,
			z: 0,
			last_update: 0,
			light_populated: false,
			terrain_populated: true,
			v: 0,
			inhabited_time: 0,
			biomes: vec![0; 256],
			heightmap: vec![0; 256],
			sections: vec![
				Section {
					y: 0,
					blocks: vec![16; 4096],
					add: None,
					data: NibbleVec::filled(),
					block_light: NibbleVec::filled(),
					sky_light: NibbleVec::filled()
				}
			],
			entities: vec![],
			tile_entities: vec![],
			tile_ticks: vec![]
		}
	};
	
	let file = File::create("/home/coderbot/r.0.0.mca").unwrap();
	let mut writer = RegionWriter::start(file).unwrap();
	
	println!("Chunk spans {} bytes", writer.chunk(0, 0, &root).unwrap());
	
	writer.finish().unwrap();
	
	let mut file = File::create("/home/coderbot/c.0.0.nbt").unwrap();
	encode::to_writer(&mut file, &root, None).unwrap();
	
	/*let trig = trig::TrigLookup::new();
	
	let vein = decorator::vein::Vein::create(32, (0, 0, 0), &mut rng, &trig);
	println!("{:?}", vein);
	
	for x in 0..33 {
		println!("{:?}", vein.blob(x, &mut rng, &trig));
	}*/
	
	/*for x in 0..400 {
		let rng = JavaRng::new(100 + x);
		
		let caves = structure::caves::Caves::for_chunk(rng, (0, 0), (0, 0));
		println!("{:?}", caves);
		
		for start in caves {
			println!("{:?}", start);
			println!("{:?}", start.to_tunnel(8));
		}
	}*/
	
	/*let table = decorator::dungeon::SimpleLootTable::default();
	
	for _ in 0..4096 {
		println!("{:?}", table.get_item(&mut rng));
	}*/
	
	/*let climate_source = ClimateSource::new(8399452073110208023);
	let climate_chunk = climate_source.chunk((-35.0 * 16.0, -117.0 * 16.0));
	
	for x in 0..16 {
		for z in 0..16 {
			let climate = climate_chunk.get(x, z);
			//println!("{:?}", climate);
		}
	}
	
	let settings = HeightSettings::default();
	println!("Settings: {:?}", settings);
	
	let mut random = JavaRng::new(8399452073110208023);
	
	// Initialize the previous noise generators.
	for _ in 0..48 {
		let p = Permutations::new(&mut random);
	}
	
	let source = HeightSource::new(&mut random, &settings);
	
	for x in 0..5 {
		for z in 0..5 {
			let climate = climate_chunk.get(x * 3 + 1, z * 3 + 1);
			println!("{:?}", source.sample(Vector2::new(-140.0 + (x as f64), -468.0 + (z as f64)), climate));
		}
	}*/
	
	//let lookup = ::biome::Lookup::generate();
	//println!("{}", lookup);
	
	//let perlin = perlin::Perlin::from_rng(&mut JavaRng::new(100), Vector3::new(0.5, 0.5, 0.5), 1.0);
	//let table = perlin.generate_y_table(0.0, 4);
	
	//println!("{:?}", perlin);
	//println!("{:.18}", perlin.generate(Vector3::new(0.0, 0.0, 0.0)));

	/*for x in 0..4 {
		for z in 0..4 {
			for y in 0..4 {
				println!("{:.16}", perlin.generate_override(Vector3::new(x as f64, y as f64, z as f64), table[y]));
			}
		}
	}*/
}