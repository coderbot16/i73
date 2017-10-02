// TODO: Remove this when i73 becomes a library.
#![allow(dead_code)]

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
mod totuple;
mod segmented;

use std::fs::File;
use nbt_serde::encode;
use chunk::grouping::{Moore, Column};
use generator::Pass;
use generator::overworld_173::{self, Settings};
use chunk::anvil::{self, ChunkRoot};
use chunk::region::RegionWriter;

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
	
	let (shape, _paint) = overworld_173::passes::<u16>(8399452073110208023, Settings::default());
	/*
	let mut moore = Moore::<u16>::with_bits(4);
	
	moore.ensure_available(0);
	moore.ensure_available(16);*/
	
	
	let file = File::create("out/region/r.0.0.mca").unwrap();
	let mut writer = RegionWriter::start(file).unwrap();
	
	for x in 0..32 {
		for z in 0..32 {
			println!("applying to {}, {}", x, z);
			
			let mut column = Column::<u16>::with_bits(4);
			
			shape.apply(&mut column, (x, z)).unwrap();
			
			let sections = column.to_anvil(vec![None; 16]).unwrap();
		
			let root = ChunkRoot {
				version: 0,
				chunk: anvil::Chunk {
					x: (x as i32),
					z: (z as i32),
					last_update: 0,
					light_populated: false,
					terrain_populated: true,
					v: 0,
					inhabited_time: 0,
					biomes: vec![0; 256],
					heightmap: vec![0; 256],
					sections,
					entities: vec![],
					tile_entities: vec![],
					tile_ticks: vec![]
				}
			};
			
			println!("Chunk spans {} bytes", writer.chunk(x as u8, z as u8, &root).unwrap());
		}
	}
	
	writer.finish().unwrap();
	
	//paint.apply(moore.column_mut(0, 0), (3, -2)).unwrap();
	
	/*
	use chunk::matcher;
	
	let lake_blocks = LakeBlocks {
		is_liquid:  matcher::None,
		is_solid:   matcher::All,
		replacable: matcher::All,
		liquid: 8*16,
		carve: 0,
		solidify: None
	};
	
	for y in 0..16 {
		moore.column_mut(0, 0).chunk_mut(y).palette_mut().replace(0, 16);
	}
	
	for y in 1..16 {
		let mut rng = JavaRng::new(100+(y as i64));
		let settings = LakeSettings::default();
		let blobs = LakeBlobs::new(&mut rng, &settings);
		let mut shape = LakeShape::new(&settings);
		shape.fill(blobs);
		
		lake_blocks.fill_and_carve(&shape, &mut moore, (0, y * 16, 0));
	}
	
	let trig_lookup = trig::TrigLookup::new();
	let vein_blocks = VeinBlocks {
		replace: always_true,
		block:   15*16
	};
	
	let mut rng = JavaRng::new(100);
	
	for _ in 0..20 {
		let (x, y, z) = (rng.next_i32(16), rng.next_i32(64), rng.next_i32(16));
		let vein = Vein::create(8, (x, y, z), &mut rng, &trig_lookup);
		
		vein_blocks.generate(&vein, &mut moore, &mut rng, &trig_lookup).unwrap();
	}*/
	
	/*let file = File::create("out/region/r.0.0.mca").unwrap();
	let mut writer = RegionWriter::start(file).unwrap();
	
	for x in 0..3 {
		for z in 0..3 {
			let sections = moore.column((x as i8) - 1, (z as i8) - 1).to_anvil(vec![None; 16]).unwrap();
		
			let root = ChunkRoot {
				version: 0,
				chunk: anvil::Chunk {
					x: (x as i32),
					z: (z as i32),
					last_update: 0,
					light_populated: false,
					terrain_populated: true,
					v: 0,
					inhabited_time: 0,
					biomes: vec![0; 256],
					heightmap: vec![0; 256],
					sections,
					entities: vec![],
					tile_entities: vec![],
					tile_ticks: vec![]
				}
			};
			
			//println!("{:?}", root);
			
			println!("Chunk spans {} bytes", writer.chunk(x, z, &root).unwrap());
			let mut file = File::create(format!("out/alpha/c.{}.{}.nbt", x, z)).unwrap();
			encode::to_writer(&mut file, &root, None).unwrap();
		}
	}
	
	writer.finish().unwrap();*/
	
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
}