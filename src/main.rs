// TODO: Remove this when i73 becomes a library.
#![allow(dead_code)]

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate nbt_serde;
extern crate byteorder;
extern crate deflate;
extern crate bit_vec;

mod noise;
mod rng;
mod biome;
mod sample;
mod noise_field;
mod decorator;
mod trig;
mod structure;
mod generator;
mod distribution;
mod chunk;
mod totuple;
mod segmented;
mod dynamics;
mod image_ops;

use std::fs::File;
use chunk::grouping::Column;
use generator::Pass;
use generator::overworld_173::{self, Settings};
use chunk::anvil::{self, ChunkRoot};
use chunk::region::RegionWriter;
use chunk::position::BlockPosition;
use biome::config::BiomesConfig;
use biome::Lookup;
use trig::TrigLookup;
use image_ops::Image;

extern crate nalgebra;

fn display_image(map: &Image<bool>) {
	for z in (0..map.z_size()).rev() {
		for x in 0..map.x_size() {
			if x == map.x_size() / 2 {
				print!("|");
			}
			
			print!("{}", if *map.get(x, z) {'#'} else {'.'});
		}
		println!();
		
		if z == map.z_size() / 2 {
			println!("======== ========");
		}
	}
}

fn main() {
	use image_ops::i80::Continents;
	use image_ops::filter::{Chain, Source, Filter};
	use image_ops::zoom::{Zoom, BestCandidate, RandomCandidate};
	use image_ops::blur::{Blur, XSpill, BoolMix};
	
	use rng::NotchRng;
	
	let continents = Continents {
		chance: 10,
		rng: NotchRng::new(1, 100)
	};
	
	let mut chain = Chain::new();
	chain.0.push(Box::new(Zoom::new(NotchRng::new(2000, 100), RandomCandidate)));
	chain.0.push(Box::new(Blur::new(NotchRng::new(   1, 100), XSpill::new(BoolMix { true_chance: 4, false_chance: 2 }))));
	chain.0.push(Box::new(Zoom::new(NotchRng::new(2001, 100), BestCandidate)));
	chain.0.push(Box::new(Blur::new(NotchRng::new(   2, 100), XSpill::new(BoolMix { true_chance: 4, false_chance: 2 }))));
	chain.0.push(Box::new(Zoom::new(NotchRng::new(2002, 100), BestCandidate)));
	chain.0.push(Box::new(Blur::new(NotchRng::new(   3, 100), XSpill::new(BoolMix { true_chance: 4, false_chance: 2 }))));
	chain.0.push(Box::new(Zoom::new(NotchRng::new(2003, 100), BestCandidate)));
	chain.0.push(Box::new(Blur::new(NotchRng::new(   3, 100), XSpill::new(BoolMix { true_chance: 4, false_chance: 2 }))));
	chain.0.push(Box::new(Zoom::new(NotchRng::new(2004, 100), BestCandidate)));
	chain.0.push(Box::new(Blur::new(NotchRng::new(   3, 100), XSpill::new(BoolMix { true_chance: 4, false_chance: 2 }))));
	
	println!("{:?} {:?}", chain.input_position((-8, -8)), chain.input_size((16, 16)));
	
	let sample = chain.input_size((16, 16));
	let mut continents_data = Image::new(false, sample.0, sample.1);
	continents.fill(chain.input_position((-8, -8)), &mut continents_data);
	
	let mut out = Image::new(false, 16, 16);
	chain.filter((-8, -8), &continents_data, &mut out);
	
	println!("Out:");
	display_image(&out);
	
	/*let zoom = Zoom::new(NotchRng::new(2000, 100), RandomCandidate);
	let sample = zoom.input_size((16, 16));
	
	let mut continents_data = Image::new(false, sample.0, sample.1);
	
	continents.fill(zoom.input_position((-8, -8)), &mut continents_data);
	
	println!("( Initial )");
	display_image(&continents_data);
	
	let mut zoomed = Image::new(false, 16, 16);
	
	zoom.filter((-8, -8), &continents_data, &mut zoomed);
	
	println!("( Zoomed )");
	display_image(&zoomed);*/
	
	
	
	/*let mut continents_data = Image::new(false, 16, 16);
	
	continents.fill((-8, -8), &mut continents_data);
	
	println!("( Initial )");
	println!("{}", continents_data);*/
	
	/*let biomes_config = serde_json::from_reader::<File, BiomesConfig>(File::open("config/biomes.json").unwrap()).unwrap();
	let grid = biomes_config.to_grid().unwrap();*/
	
	/*let reduction_table = nether_173::generate_reduction_table(17);
	for reduction in &reduction_table {
		println!("{}", reduction);
	}*/
	
	/*use dynamics::light::{Lighting, SkyLightSources, Meta};
	use dynamics::queue::{Queue, LayerMask};
	
	let mut column = Column::<u16>::with_bits(4);
	column.chunk_mut(0).palette_mut().replace(0,  0 * 16);
	column.chunk_mut(0).palette_mut().replace(1, 89 * 16);
	column.chunk_mut(0).palette_mut().replace(2,  1 * 16);
	column.chunk_mut(0).set_immediate(BlockPosition::new(7, 7, 7), &(89 * 16));
	
	{
		let (blocks, palette) = column.chunk_mut(0).freeze_palette();
		
		let stone = palette.reverse_lookup(&16).unwrap();
		
		for x in 0..1792 {
			let pos = BlockPosition::from_yzx(x);
			
			blocks.set(pos, &stone);
		}
	}*/
	
	/*while light.step(&column.chunk(0), &mut queue) {
		println!("S {:?}", light);
	}
	
	println!("-- done --");
	
	let file = File::create("out/region/r.0.0.mca").unwrap();
	let mut writer = RegionWriter::start(file).unwrap();
	
	let mut heightmap = (Box::new(light::generate_heightmap(&column, &0)) as Box<[u32]>).into_vec();
	
	{
		{
			let (blocks, data, add) = column.chunk(0).to_anvil().unwrap();
			
			let root = ChunkRoot {
				version: 0,
				chunk: anvil::Chunk {
					x: 0,
					z: 0,
					last_update: 0,
					light_populated: false,
					terrain_populated: true,
					v: 0,
					inhabited_time: 0,
					biomes: vec![0; 256],
					heightmap,
					sections: vec![anvil::Section {
						y: 0,
						blocks,
						add,
						data,
						block_light: light.to_anvil(),
						sky_light: anvil::NibbleVec::filled()
					}],
					entities: vec![],
					tile_entities: vec![],
					tile_ticks: vec![]
				}
			};
			
			println!("Chunk spans {} bytes", writer.chunk(0, 0, &root).unwrap());
		}
	}
	
	writer.finish().unwrap();*/
	
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

	/*let mut lighting_info = ::std::collections::HashMap::new();
	lighting_info.insert( 0 * 16, Meta::new(0));
	lighting_info.insert( 8 * 16, Meta::new(2));
	lighting_info.insert( 9 * 16, Meta::new(2));
	
	let (shape, paint) = overworld_173::passes(8399452073110208023, Settings::default(), Lookup::generate(&grid));
	
	let caves_generator = structure::caves::CavesGenerator { 
		lookup: TrigLookup::new(), 
		carve: 0*16,
		ocean: |ty: &u16| -> bool {
			*ty == 8*16 || *ty == 9*16
		},
		carvable: |ty: &u16| -> bool {
			*ty == 1*16 || *ty == 2*16 || *ty == 3*16
		}
	};
	let caves = structure::StructureGenerateNearby::new(8399452073110208023, 8, caves_generator);*/
	
	/*let shape = nether_173::passes(-160654125608861039, &nether_173::default_tri_settings(), nether_173::ShapeBlocks::default(), 31);
	
	let default_grid = biome::default_grid();
	
	let mut fake_settings = Settings::default();
	fake_settings.biome_lookup = biome::Lookup::filled(default_grid.lookup(biome::climate::Climate::new(0.5, 0.0)));
	fake_settings.sea_coord = 31;
	fake_settings.beach = None;
	fake_settings.max_bedrock_height = None;
	
	let (_, paint) = overworld_173::passes(-160654125608861039, fake_settings);*/
	
	/*let file = File::create("out/region/r.0.0.mca").unwrap();
	let mut writer = RegionWriter::start(file).unwrap();
	
	for x in 0..32 {
		for z in 0..32 {
			println!("applying to {}, {}", x, z);
			
			let mut column = Column::<u16>::with_bits(4);
			
			shape.apply(&mut column, (x, z)).unwrap();
			paint.apply(&mut column, (x, z)).unwrap();
			caves.apply(&mut column, (x, z)).unwrap();
			
			let mut column_light = vec![None; 16];
			
			let mut mask = LayerMask::default();
			
			for y in (0..16).rev() {
				let chunk = column.chunk(y);
				
				let mut meta = Vec::with_capacity(chunk.palette().entries().len());
				
				for value in chunk.palette().entries() {
					if let &Some(ref entry) = value {
						meta.push(lighting_info.get(entry).map(|&meta| meta).unwrap_or(Meta::new(15)))
					} else {
						meta.push(Meta::new(15))
					}
				}
				
				let sources = SkyLightSources::build(chunk, &meta, mask);
		
				let mut queue = Queue::new();
				let mut light = Lighting::new(sources, meta);
				
				light.initial(chunk, &mut queue);
				light.finish(chunk, &mut queue);
				
				// TODO: Inter chunk lighting interactions.
			
				let (light_data, sources) = light.decompose();
				mask = sources.into_mask();
			
				column_light[y] = Some((chunk::anvil::NibbleVec::filled(), light_data.to_anvil()));
			}
			
			let sections = column.to_anvil(column_light).unwrap();
		
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
			
			println!("Chunk spans {} bytes", writer.chunk(x as u8, (z) as u8, &root).unwrap());
		}
	}
	
	writer.finish().unwrap();*/
	
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