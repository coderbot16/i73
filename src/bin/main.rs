extern crate rs25;
extern crate vocs;
extern crate i73;
extern crate serde_json;

use std::path::PathBuf;
use std::fs::File;
use std::cmp::min;

use i73::config::settings::customized::{Customized, Parts};
use i73::generator::Pass;
use i73::generator::overworld_173::{self, Settings};
use i73::config::biomes::BiomesConfig;
use i73::biome::Lookup;
use i73::structure;

use vocs::indexed::ChunkIndexed;
use vocs::world::world::World;
use vocs::view::ColumnMut;
use vocs::position::GlobalColumnPosition;

use rs25::level::manager::{ColumnSnapshot, ChunkSnapshot};
use rs25::level::region::RegionWriter;
use rs25::level::anvil::ColumnRoot;

// use i73::image_ops::Image;
/*fn display_image(map: &Image<bool>) {
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
}*/

fn main() {
	let profile_name = match ::std::env::args().skip(1).next() {
		Some(name) => name,
		None => {
			println!("Usage: i73 <profile>");
			return;
		}
	};
	
	let mut profile = PathBuf::new();
	profile.push("profiles");
	profile.push(&profile_name);
	
	println!("Using profile {}: {}", profile_name, profile.to_string_lossy());
	
	let customized = serde_json::from_reader::<File, Customized>(File::open(profile.join("customized.json")).unwrap()).unwrap();
	let parts = Parts::from(customized);
	
	println!("  Tri Noise Settings: {:?}", parts.tri);
	println!("  Height Stretch: {:?}", parts.height_stretch);
	println!("  Height Settings: {:?}", parts.height);
	println!("  Biome Settings: {:?}", parts.biome);
	println!("  Structures: {:?}", parts.structures);
	println!("  Decorators: {:?}", parts.decorators);
	
	let mut settings = Settings::default();
	
	settings.tri = parts.tri;
	settings.height = parts.height.into();
	settings.field.height_stretch = parts.height_stretch;
	
	// TODO: Biome Settings
	
	let sea_block = if parts.ocean.top > 0 {
		settings.sea_coord = min(parts.ocean.top - 1, 255) as u8;
		
		if parts.ocean.lava { 11*16 } else { 9*16 }
	} else {
		0*16
	};
	
	settings.shape_blocks.ocean = sea_block;
	settings.paint_blocks.ocean = sea_block;
	
	// TODO: Structures and Decorators
	
	/*use image_ops::i80::Continents;
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
	display_image(&out);*/
	
	let biomes_config = serde_json::from_reader::<File, BiomesConfig>(File::open(profile.join("biomes.json")).unwrap()).unwrap();
	let grid = biomes_config.to_grid().unwrap();
	
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

	let mut lighting_info = ::std::collections::HashMap::new();
	lighting_info.insert( 0 * 16, Meta::new(0));
	lighting_info.insert( 8 * 16, Meta::new(2));
	lighting_info.insert( 9 * 16, Meta::new(2));
	
	let (shape, paint) = overworld_173::passes(8399452073110208023, settings, Lookup::generate(&grid));
	
	let caves_generator = structure::caves::CavesGenerator {
		carve: 0*16,
		lower: 10*16,
		surface_block: 2*16,
		ocean: |ty: &u16| -> bool {
			*ty == 8*16 || *ty == 9*16
		},
		carvable: |ty: &u16| -> bool {
			*ty == 1*16 || *ty == 2*16 || *ty == 3*16
		},
		surface_top: |ty: &u16| -> bool {
			*ty == 2*16
		},
		surface_fill: |ty: &u16| -> bool {
			*ty == 3*16
		},
		blob_size_multiplier: 1.0,
		vertical_multiplier: 1.0,
		lower_surface: 10
	};
	let caves = structure::StructureGenerateNearby::new(8399452073110208023, 8, caves_generator);
	
	/*let shape = nether_173::passes(-160654125608861039, &nether_173::default_tri_settings(), nether_173::ShapeBlocks::default(), 31);
	
	let default_grid = biome::default_grid();
	
	let mut fake_settings = Settings::default();
	fake_settings.biome_lookup = biome::Lookup::filled(default_grid.lookup(biome::climate::Climate::new(0.5, 0.0)));
	fake_settings.sea_coord = 31;
	fake_settings.beach = None;
	fake_settings.max_bedrock_height = None;
	
	let (_, paint) = overworld_173::passes(-160654125608861039, fake_settings);*/
	
	let mut world = World::<ChunkIndexed<u16>>::new();
	
	println!("Generating region (0, 0)");

	for x in 0..32 {
		println!("{}", x);
		for z in 0..32 {
			let column_position = GlobalColumnPosition::new(x, z);

			let mut column_chunks = [
				ChunkIndexed::<u16>::new(4, 0),
				ChunkIndexed::<u16>::new(4, 0),
				ChunkIndexed::<u16>::new(4, 0),
				ChunkIndexed::<u16>::new(4, 0),
				ChunkIndexed::<u16>::new(4, 0),
				ChunkIndexed::<u16>::new(4, 0),
				ChunkIndexed::<u16>::new(4, 0),
				ChunkIndexed::<u16>::new(4, 0),
				ChunkIndexed::<u16>::new(4, 0),
				ChunkIndexed::<u16>::new(4, 0),
				ChunkIndexed::<u16>::new(4, 0),
				ChunkIndexed::<u16>::new(4, 0),
				ChunkIndexed::<u16>::new(4, 0),
				ChunkIndexed::<u16>::new(4, 0),
				ChunkIndexed::<u16>::new(4, 0),
				ChunkIndexed::<u16>::new(4, 0)
			];

			{
				let mut column: ColumnMut<u16> = ColumnMut::from_array(&mut column_chunks);

				shape.apply(&mut column, column_position);
				paint.apply(&mut column, column_position);
				caves.apply(&mut column, column_position);
			}

			world.set_column((x as i32, z as i32), column_chunks);
		}
	}

	println!("Decorating region (0, 0)");

	let mut decoration_rng = ::i73::rng::JavaRng::new(8399452073110208023);
	let coefficients = (
		((decoration_rng.next_i64() >> 1) << 1) + 1,
		((decoration_rng.next_i64() >> 1) << 1) + 1
	);

	let gravel_vein = ::i73::decorator::Dispatcher {
		decorator: ::i73::decorator::vein::VeinDecorator {
			blocks: ::i73::decorator::vein::VeinBlocks {
				replace: |ty: &u16| -> bool {
					*ty == 1*16
				},
				block: 13*16
			},
			size: 32
		},
		height_distribution: ::i73::distribution::height::Linear {
			min: 0,
			max: 63
		},
		rarity: ::i73::distribution::rarity::Common {
			max: 9
		},
		phantom: ::std::marker::PhantomData::<u16>
	};

	let clay_vein = ::i73::decorator::Dispatcher {
		decorator: ::i73::decorator::vein::SeasideVeinDecorator {
			vein: ::i73::decorator::vein::VeinDecorator {
				blocks: ::i73::decorator::vein::VeinBlocks {
					replace: |ty: &u16| -> bool {
						*ty == 12*16
					},
					block: 82*16
				},
				size: 32
			},
			ocean: |ty: &u16| -> bool {
				*ty == 8*16 || *ty == 9*16
			}
		},
		height_distribution: ::i73::distribution::height::Linear {
			min: 0,
			max: 63
		},
		rarity: ::i73::distribution::rarity::Common {
			max: 9
		},
		phantom: ::std::marker::PhantomData::<u16>
	};

	let water_lake = ::i73::decorator::Dispatcher {
		decorator: ::i73::decorator::lake::LakeDecorator {
			blocks: ::i73::decorator::lake::LakeBlocks {
				is_liquid:  |ty: &u16| -> bool { *ty == 8*16 || *ty == 9*16 || *ty == 10*16 || *ty == 11*16 },
				is_solid:   |ty: &u16| -> bool { !(*ty == 0 || *ty == 8*16 || *ty == 9*16 || *ty == 10*16 || *ty == 11*16) }, // TODO: All nonsolid blocks
				replacable: |_: &u16| -> bool { unimplemented!() },
				liquid:     9*16,
				carve:      0*16,
				solidify:   None
			},
			settings: ::i73::decorator::lake::LakeSettings::default()
		},
		height_distribution: ::i73::distribution::height::Linear {
			min: 0,
			max: 127
		},
		rarity: ::i73::distribution::rarity::Rare {
			base: 1,
			rarity: 4
		},
		phantom: ::std::marker::PhantomData::<u16>
	};

	for x in 0..31 {
		println!("{}", x);
		for z in 0..31 {
			let x_part = (x as i64).wrapping_mul(coefficients.0);
			let z_part = (z as i64).wrapping_mul(coefficients.1);
			decoration_rng = ::i73::rng::JavaRng::new((x_part.wrapping_add(z_part)) ^ 8399452073110208023);

			let mut quad = world.get_quad_mut((x as i32, z as i32)).unwrap();

			water_lake.generate(&mut quad, &mut decoration_rng).unwrap();
			clay_vein.generate(&mut quad, &mut decoration_rng).unwrap();
			gravel_vein.generate(&mut quad, &mut decoration_rng).unwrap();
		}
	}

	use vocs::nibbles::ChunkNibbles;
	use vocs::mask::LayerMask;
	use rs25::dynamics::light::{Meta, SkyLightSources, Lighting, HeightMapBuilder};
	use rs25::dynamics::queue::Queue;

	println!("Writing region (0, 0)");

	// use rs25::level::manager::{Manager, RegionPool};
	// let pool = RegionPool::new(PathBuf::from("out/region/"), 512);
	// let mut manager = Manager::manage(pool);

	let file = File::create("out/region/r.0.0.mca").unwrap();
	let mut writer = RegionWriter::start(file).unwrap();

	for z in 0..32 {
		println!("{}", z);
		for x in 0..32 {

			let column = ColumnMut(world.get_column_mut((x as i32, z as i32)).unwrap());

			let mut snapshot_light = vec![None; 16];

			let mut mask = LayerMask::default();
			let mut heightmap = HeightMapBuilder::new();

			for y in (0..16).rev() {
				let chunk = &column.0[y];

				let mut meta = Vec::with_capacity(chunk.palette().entries().len());

				for value in chunk.palette().entries() {
					if let &Some(ref entry) = value {
						meta.push(lighting_info.get(entry).map(|&meta| meta).unwrap_or(Meta::new(15)))
					} else {
						meta.push(Meta::new(15))
					}
				}

				let sources = SkyLightSources::build(chunk.freeze().0, &meta, mask);

				let mut queue = Queue::default();
				let mut light = Lighting::new(sources, meta);

				light.initial(chunk.freeze().0, &mut queue);
				light.finish(chunk.freeze().0, &mut queue);

				// TODO: Inter chunk lighting interactions.

				let (light_data, sources) = light.decompose();
				mask = heightmap.add(sources);

				snapshot_light[y] = Some((ChunkNibbles::new(), light_data));
			}

			let mut snapshot = ColumnSnapshot {
				chunks: vec![None; 16],
				last_update: 0,
				light_populated: true,
				terrain_populated: true,
				inhabited_time: 0,
				biomes: vec![0; 256],
				heightmap: heightmap.build().into_vec(),
				entities: vec![],
				tile_entities: vec![],
				tile_ticks: vec![]
			};

			for y in 0..16 {
				if column.0[y].anvil_empty() {
					continue;
				}

				let snapshot_light = snapshot_light[y].take().unwrap_or_else(|| (ChunkNibbles::new(), ChunkNibbles::new()));

				snapshot.chunks[y] = Some(ChunkSnapshot {
					blocks: column.0[y].clone(),
					block_light: snapshot_light.0,
					sky_light: snapshot_light.1
				});
			};

			let root = ColumnRoot::from(snapshot.to_column(x as i32, z as i32).unwrap());

			writer.chunk(x as u8, z as u8, &root).unwrap();
		}
	}
	
	writer.finish().unwrap();
}