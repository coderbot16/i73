extern crate rs25;
extern crate vocs;
extern crate i73;
extern crate java_rand;
#[macro_use]
extern crate serde_json;

use std::path::PathBuf;
use std::fs::File;
use std::cmp::min;

use i73::config::settings::customized::{Customized, Parts};
use i73::generator::Pass;
use i73::generator::overworld_173::{self, Settings};
use i73::config::biomes::{BiomesConfig, DecoratorConfig};
use i73::biome::Lookup;
use i73::structure;
use i73::matcher::BlockMatcher;

use vocs::indexed::ChunkIndexed;
use vocs::world::world::World;
use vocs::view::ColumnMut;
use vocs::position::GlobalColumnPosition;

use rs25::level::manager::{ColumnSnapshot, ChunkSnapshot};
use rs25::level::region::RegionWriter;
use rs25::level::anvil::ColumnRoot;

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
	
	let biomes_config = serde_json::from_reader::<File, BiomesConfig>(File::open(profile.join("biomes.json")).unwrap()).unwrap();
	let grid = biomes_config.to_grid().unwrap();

	let mut decorator_registry: ::std::collections::HashMap<String, Box<i73::decorator::DecoratorFactory<u16>>> = ::std::collections::HashMap::new();
	decorator_registry.insert("vein".into(), Box::new(::i73::decorator::vein::VeinDecoratorFactory::default()));
	decorator_registry.insert("seaside_vein".into(), Box::new(::i73::decorator::vein::SeasideVeinDecoratorFactory::default()));

	let gravel_config = DecoratorConfig {
		decorator: "vein".into(),
		settings: json!({
			"blocks": {
				"replace": {
					"blacklist": false,
					"blocks": [16]
				},
				"block": 208
			},
			"size": 32
		}),
		height_distribution: ::i73::distribution::Chance {
			base: i73::distribution::Baseline::Linear(i73::distribution::Linear {
				min: 0,
				max: 63
			}),
			ordering: i73::distribution::ChanceOrdering::AlwaysGeneratePayload,
			chance: 1
		},
		count: ::i73::distribution::Chance {
			base: i73::distribution::Baseline::Linear(i73::distribution::Linear {
				min: 0,
				max: 9
			}),
			ordering: i73::distribution::ChanceOrdering::AlwaysGeneratePayload,
			chance: 1
		}
	};

	let mut decorators: Vec<::i73::decorator::Dispatcher<i73::distribution::Chance<i73::distribution::Baseline>, i73::distribution::Chance<i73::distribution::Baseline>, u16>> = Vec::new();

	decorators.push (::i73::decorator::Dispatcher {
		decorator: Box::new(::i73::decorator::lake::LakeDecorator {
			blocks: ::i73::decorator::lake::LakeBlocks {
				is_liquid:  BlockMatcher::include([8*16, 9*16, 10*16, 11*16].iter()),
				is_solid:   BlockMatcher::exclude([0*16, 8*16, 9*16, 10*16, 11*16].iter()), // TODO: All nonsolid blocks
				replacable: BlockMatcher::none(), // TODO
				liquid:     9*16,
				carve:      0*16,
				solidify:   None
			},
			settings: ::i73::decorator::lake::LakeSettings::default()
		}),
		height_distribution: ::i73::distribution::Chance {
			base: i73::distribution::Baseline::Linear(i73::distribution::Linear {
				min: 0,
				max: 127
			}),
			ordering: i73::distribution::ChanceOrdering::AlwaysGeneratePayload,
			chance: 1
		},
		rarity: ::i73::distribution::Chance {
			base: ::i73::distribution::Baseline::Constant { value: 1 },
			chance: 4,
			ordering: ::i73::distribution::ChanceOrdering::AlwaysGeneratePayload
		}
	});

	decorators.push (::i73::decorator::Dispatcher {
		decorator: Box::new(::i73::decorator::vein::SeasideVeinDecorator {
			vein: ::i73::decorator::vein::VeinDecorator {
				blocks: ::i73::decorator::vein::VeinBlocks {
					replace: BlockMatcher::is(12*16),
					block: 82*16
				},
				size: 32
			},
			ocean: BlockMatcher::include([8*16, 9*16].iter())
		}),
		height_distribution: ::i73::distribution::Chance {
			base: i73::distribution::Baseline::Linear(i73::distribution::Linear {
				min: 0,
				max: 63
			}),
			ordering: i73::distribution::ChanceOrdering::AlwaysGeneratePayload,
			chance: 1
		},
		rarity: ::i73::distribution::Chance {
			base: i73::distribution::Baseline::Linear(i73::distribution::Linear {
				min: 0,
				max: 9
			}),
			ordering: i73::distribution::ChanceOrdering::AlwaysGeneratePayload,
			chance: 1
		}
	});

	decorators.push (gravel_config.into_dispatcher(&decorator_registry).unwrap());

	decorators.push (::i73::decorator::Dispatcher {
		decorator: Box::new(::i73::decorator::clump::Clump {
			iterations: 64,
			horizontal: 8,
			vertical: 4,
			decorator: ::i73::decorator::clump::plant::PlantDecorator {
				block: 31*16 + 1,
				base: BlockMatcher::include([2*16, 3*16, 60*16].into_iter()),
				replace: BlockMatcher::is(0*16)
			},
			phantom: ::std::marker::PhantomData::<u16>
		}),
		height_distribution: ::i73::distribution::Chance {
			base: i73::distribution::Baseline::Linear(i73::distribution::Linear {
				min: 0,
				max: 127
			}),
			ordering: i73::distribution::ChanceOrdering::AlwaysGeneratePayload,
			chance: 1
		},
		rarity: ::i73::distribution::Chance {
			base: i73::distribution::Baseline::Linear(i73::distribution::Linear {
				min: 0,
				max: 90
			}),
			ordering: i73::distribution::ChanceOrdering::AlwaysGeneratePayload,
			chance: 1
		}
	});

	/*use decorator::large_tree::{LargeTreeSettings, LargeTree};
	let settings = LargeTreeSettings::default();
	
	for i in 0..1 {
		let mut rng = Random::new(100 + i);
		let shape = settings.tree((0, 0, 0), &mut rng, None, 20);
		
		println!("{:?}", shape);
		
		let mut y = shape.foliage_max_y - 1;
		while y >= shape.foliage_min_y {
			let spread = shape.spread(y);
			
			println!("y: {}, spread: {}", y, spread);
			
			for _ in 0..shape.foliage_per_y {
				println!("{:?}", shape.foliage(y, spread, &mut rng));
			}
			
			y -= 1;
		}
	}*/
	
	let (shape, paint) = overworld_173::passes(8399452073110208023, settings, Lookup::generate(&grid));
	
	let caves_generator = structure::caves::CavesGenerator {
		carve: 0*16,
		lower: 10*16,
		surface_block: 2*16,
		ocean: BlockMatcher::include([8*16, 9*16].iter()),
		carvable: BlockMatcher::include([1*16, 2*16, 3*16].iter()),
		surface_top: BlockMatcher::is(2*16),
		surface_fill: BlockMatcher::is(3*16),
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
	let gen_start = ::std::time::Instant::now();

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

	{
		let end = ::std::time::Instant::now();
		let time = end.duration_since(gen_start);

		let secs = time.as_secs();
		let us = (secs * 1000000) + ((time.subsec_nanos() / 1000) as u64);

		println!("Generation done in {}us ({}us per column)", us, us / 1024);
	}

	println!("Decorating region (0, 0)");
	let dec_start = ::std::time::Instant::now();

	let mut decoration_rng = ::java_rand::Random::new(8399452073110208023);
	let coefficients = (
		((decoration_rng.next_i64() >> 1) << 1) + 1,
		((decoration_rng.next_i64() >> 1) << 1) + 1
	);

	for x in 0..31 {
		println!("{}", x);
		for z in 0..31 {
			let x_part = (x as i64).wrapping_mul(coefficients.0) as u64;
			let z_part = (z as i64).wrapping_mul(coefficients.1) as u64;
			decoration_rng = ::java_rand::Random::new((x_part.wrapping_add(z_part)) ^ 8399452073110208023);

			let mut quad = world.get_quad_mut((x as i32, z as i32)).unwrap();

			for dispatcher in &decorators {
				dispatcher.generate(&mut quad, &mut decoration_rng).unwrap();
			}
		}
	}

	{
		let end = ::std::time::Instant::now();
		let time = end.duration_since(dec_start);

		let secs = time.as_secs();
		let us = (secs * 1000000) + ((time.subsec_nanos() / 1000) as u64);

		println!("Decoration done in {}us ({}us per column)", us, us / 1024);
	}

	use vocs::nibbles::{u4, ChunkNibbles, BulkNibbles};
	use vocs::sparse::SparseStorage;
	use vocs::mask::LayerMask;
	use rs25::dynamics::light::{SkyLightSources, Lighting, HeightMapBuilder};
	use rs25::dynamics::queue::Queue;

	let mut sky_light = World::<ChunkNibbles>::new();
	let mut heightmaps = ::std::collections::HashMap::<(i32, i32), Vec<u32>>::new(); // TODO: Better vocs integration.

	let mut lighting_info = SparseStorage::<u4>::with_default(u4::new(15));
	lighting_info.set( 0 * 16, u4::new(0));
	lighting_info.set( 8 * 16, u4::new(2));
	lighting_info.set( 9 * 16, u4::new(2));

	let mut queue = Queue::default();

	println!("Performing initial sky lighting for region (0, 0)");
	let lighting_start = ::std::time::Instant::now();

	for x in 0..32 {
		println!("{}", x);
		for z in 0..32 {
			let column = ColumnMut(world.get_column_mut((x as i32, z as i32)).unwrap());

			let mut mask = LayerMask::default();
			let mut heightmap = HeightMapBuilder::new();

			for y in (0..16).rev() {
				let (blocks, palette) = column.0[y].freeze();

				let mut opacity = BulkNibbles::new(palette.len());

				for (index, value) in palette.iter().enumerate() {
					opacity.set(index, value.map(|entry| lighting_info.get(entry as usize)).unwrap_or(lighting_info.default_value()));
				}

				let sources = SkyLightSources::build(blocks, &opacity, mask);

				let mut light = Lighting::new(sources, opacity);

				light.initial(blocks, &mut queue);
				light.finish(blocks, &mut queue);

				// TODO: Inter chunk lighting interactions.

				let (light_data, sources) = light.decompose();
				mask = heightmap.add(sources);

				sky_light.set((x, y as u8, z), light_data);
			}

			heightmaps.insert((x, z), heightmap.build().into_vec());
		}
	}

	{
		let end = ::std::time::Instant::now();
		let time = end.duration_since(lighting_start);

		let secs = time.as_secs();
		let us = (secs * 1000000) + ((time.subsec_nanos() / 1000) as u64);

		println!("Initial sky lighting done in {}us ({}us per column)", us, us / 1024);
	}

	println!("Writing region (0, 0)");
	let writing_start = ::std::time::Instant::now();

	// use rs25::level::manager::{Manager, RegionPool};
	// let pool = RegionPool::new(PathBuf::from("out/region/"), 512);
	// let mut manager = Manager::manage(pool);

	let file = File::create("out/region/r.0.0.mca").unwrap();
	let mut writer = RegionWriter::start(file).unwrap();

	for z in 0..32 {
		println!("{}", z);
		for x in 0..32 {
			let heightmap = heightmaps.remove(&(x, z)).unwrap();
			let column = ColumnMut(world.get_column_mut((x as i32, z as i32)).unwrap());

			let mut snapshot = ColumnSnapshot {
				chunks: vec![None; 16],
				last_update: 0,
				light_populated: true,
				terrain_populated: true,
				inhabited_time: 0,
				biomes: vec![0; 256],
				heightmap,
				entities: vec![],
				tile_entities: vec![],
				tile_ticks: vec![]
			};

			for y in 0..16 {
				if column.0[y].anvil_empty() {
					continue;
				}

				let sky_light = sky_light.remove((x, y as u8, z)).unwrap()/*_or_else(ChunkNibbles::default)*/;

				snapshot.chunks[y] = Some(ChunkSnapshot {
					blocks: column.0[y].clone(),
					block_light: ChunkNibbles::default(),
					sky_light
				});
			};

			let root = ColumnRoot::from(snapshot.to_column(x as i32, z as i32).unwrap());

			writer.chunk(x as u8, z as u8, &root).unwrap();
		}
	}
	
	writer.finish().unwrap();

	{
		let end = ::std::time::Instant::now();
		let time = end.duration_since(writing_start);

		let secs = time.as_secs();
		let us = (secs * 1000000) + ((time.subsec_nanos() / 1000) as u64);

		println!("Writing done in {}us ({}us per column)", us, us / 1024);
	}
}