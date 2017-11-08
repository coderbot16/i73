use noise_field::volume::TriNoiseSettings;
use noise_field::height::HeightSettings81;
use nalgebra::Vector3;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Customized {
	#[serde(rename="coordinateScale")]			pub coordinate_scale: 			f64,
	#[serde(rename="heightScale")]				pub height_scale: 				f64,
	
	#[serde(rename="lowerLimitScale")]			pub lower_limit_scale: 			f64,
	#[serde(rename="upperLimitScale")]			pub upper_limit_scale: 			f64,
	
	#[serde(rename="mainNoiseScaleX")]			pub main_noise_scale_x: 		f64,
	#[serde(rename="mainNoiseScaleY")]			pub main_noise_scale_y: 		f64,
	#[serde(rename="mainNoiseScaleZ")]			pub main_noise_scale_z: 		f64,
	
	#[serde(rename="depthNoiseScaleX")]			pub depth_noise_scale_x: 		f64,
	#[serde(rename="depthNoiseScaleZ")]			pub depth_noise_scale_z: 		f64,
	#[serde(rename="depthNoiseScaleExponent")]	pub depth_noise_scale_exponent: f64, // Unused.
	
	#[serde(rename="baseSize")]					pub depth_base: 				f64,
	#[serde(rename="stretchY")]					pub height_stretch: 			f64,
	
	#[serde(rename="biomeDepthWeight")]			pub biome_depth_weight: 		f64,
	#[serde(rename="biomeDepthOffset")]			pub biome_depth_offset: 		f64,
	#[serde(rename="biomeScaleWeight")]			pub biome_scale_weight: 		f64,
	#[serde(rename="biomeScaleOffset")]			pub biome_scale_offset: 		f64,
	
	#[serde(rename="seaLevel")]					pub sea_level: 					i32,
	#[serde(rename="useCaves")]					pub use_caves: 					bool,
	#[serde(rename="useDungeons")]				pub use_dungeons: 				bool,
	#[serde(rename="dungeonChance")]			pub dungeon_chance: 			i32,
	#[serde(rename="useStrongholds")]			pub use_strongholds: 			bool,
	#[serde(rename="useVillages")]				pub use_villages: 				bool,
	#[serde(rename="useMineShafts")]			pub use_mineshafts: 			bool,
	#[serde(rename="useTemples")]				pub use_temples: 				bool,
	#[serde(rename="useRavines")]				pub use_ravines: 				bool,
	#[serde(rename="useWaterLakes")]			pub use_water_lakes: 			bool,
	#[serde(rename="waterLakeChance")]			pub water_lake_chance: 			i32,
	#[serde(rename="useLavaLakes")]				pub use_lava_lakes: 			bool,
	#[serde(rename="lavaLakeChance")]			pub lava_lake_chance: 			i32,
	#[serde(rename="useLavaOceans")]			pub use_lava_oceans: 			bool,
	
	#[serde(rename="fixedBiome")]				pub fixed_biome: 				i32,
	#[serde(rename="biomeSize")]				pub biome_size: 				i32,
	#[serde(rename="riverSize")]				pub river_size: 				i32,
	
	#[serde(rename="dirtSize")]					pub dirt_size:	 				i32,
	#[serde(rename="dirtCount")]				pub dirt_count:	 				i32,
	#[serde(rename="dirtMinHeight")]			pub dirt_min_height:	 		i32,
	#[serde(rename="dirtMaxHeight")]			pub dirt_max_height:	 		i32,
	
	#[serde(rename="gravelSize")]				pub gravel_size:	 			i32,
	#[serde(rename="gravelCount")]				pub gravel_count:	 			i32,
	#[serde(rename="gravelMinHeight")]			pub gravel_min_height:	 		i32,
	#[serde(rename="gravelMaxHeight")]			pub gravel_max_height:	 		i32,
	
	#[serde(rename="graniteSize")]				pub granite_size:	 			i32,
	#[serde(rename="graniteCount")]				pub granite_count:	 			i32,
	#[serde(rename="graniteMinHeight")]			pub granite_min_height:	 		i32,
	#[serde(rename="graniteMaxHeight")]			pub granite_max_height:	 		i32,
	
	#[serde(rename="dioriteSize")]				pub diorite_size:	 			i32,
	#[serde(rename="dioriteCount")]				pub diorite_count:	 			i32,
	#[serde(rename="dioriteMinHeight")]			pub diorite_min_height:	 		i32,
	#[serde(rename="dioriteMaxHeight")]			pub diorite_max_height:	 		i32,
	
	#[serde(rename="andesiteSize")]				pub andesite_size:	 			i32,
	#[serde(rename="andesiteCount")]			pub andesite_count:	 			i32,
	#[serde(rename="andesiteMinHeight")]		pub andesite_min_height:	 	i32,
	#[serde(rename="andesiteMaxHeight")]		pub andesite_max_height:	 	i32,
	
	#[serde(rename="coalSize")]					pub coal_size:	 				i32,
	#[serde(rename="coalCount")]				pub coal_count:	 				i32,
	#[serde(rename="coalMinHeight")]			pub coal_min_height:	 		i32,
	#[serde(rename="coalMaxHeight")]			pub coal_max_height:	 		i32,
	
	#[serde(rename="ironSize")]					pub iron_size:	 				i32,
	#[serde(rename="ironCount")]				pub iron_count:	 				i32,
	#[serde(rename="ironMinHeight")]			pub iron_min_height:	 		i32,
	#[serde(rename="ironMaxHeight")]			pub iron_max_height:	 		i32,
	
	#[serde(rename="goldSize")]					pub gold_size:	 				i32,
	#[serde(rename="goldCount")]				pub gold_count:	 				i32,
	#[serde(rename="goldMinHeight")]			pub gold_min_height:	 		i32,
	#[serde(rename="goldMaxHeight")]			pub gold_max_height:	 		i32,
	
	#[serde(rename="redstoneSize")]				pub redstone_size:	 			i32,
	#[serde(rename="redstoneCount")]			pub redstone_count:	 			i32,
	#[serde(rename="redstoneMinHeight")]		pub redstone_min_height:	 	i32,
	#[serde(rename="redstoneMaxHeight")]		pub redstone_max_height:	 	i32,
	
	#[serde(rename="diamondSize")]				pub diamond_size:	 			i32,
	#[serde(rename="diamondCount")]				pub diamond_count:	 			i32,
	#[serde(rename="diamondMinHeight")]			pub diamond_min_height:		 	i32,
	#[serde(rename="diamondMaxHeight")]			pub diamond_max_height:		 	i32,
	
	#[serde(rename="lapisSize")]				pub lapis_size:	 				i32,
	#[serde(rename="lapisCount")]				pub lapis_count:	 			i32,
	#[serde(rename="lapisCenterHeight")]		pub lapis_center_height:		i32,
	#[serde(rename="lapisSpread")]				pub lapis_spread:		 		i32
}

#[derive(Debug, PartialEq)]
pub struct Parts {
	pub tri:            TriNoiseSettings,
	pub height_stretch: f64,
	pub height:         HeightSettings81,
	pub biome:          BiomeSettings,
	pub ocean:          Ocean,
	pub structures:     Structures,
	pub decorators:     Decorators
}

impl From<Customized> for Parts {
	fn from(settings: Customized) -> Self {
		let h_scale = settings.coordinate_scale;
		let y_scale = settings.height_scale;
		
		Parts {
			tri: TriNoiseSettings {
				 main_out_scale:  20.0,
				upper_out_scale: settings.upper_limit_scale,
				lower_out_scale: settings.lower_limit_scale,
				lower_scale:     Vector3::new(h_scale,                               y_scale,                               h_scale                              ),
				upper_scale:     Vector3::new(h_scale,                               y_scale,                               h_scale                              ),
				 main_scale:     Vector3::new(h_scale / settings.main_noise_scale_x, y_scale / settings.main_noise_scale_y, h_scale / settings.main_noise_scale_z),
				y_size:          17
			},
			height_stretch: settings.height_stretch,
			height: HeightSettings81 {
				coord_scale: Vector3::new(settings.depth_noise_scale_x, 0.0, settings.depth_noise_scale_z),
				out_scale:   8000.0,
				base:        settings.depth_base
			},
			biome: BiomeSettings {
				depth_weight: settings.biome_depth_weight,
				depth_offset: settings.biome_depth_offset,
				scale_weight: settings.biome_scale_weight,
				scale_offset: settings.biome_scale_offset,
				fixed:        settings.fixed_biome,
				biome_size:   settings.biome_size,
				river_size:   settings.river_size
			},
			ocean: Ocean {
				top:  settings.sea_level,
				lava: settings.use_lava_oceans
			},
			structures: Structures {
				caves:       settings.use_caves,
				strongholds: settings.use_strongholds,
				villages:    settings.use_villages,
				mineshafts:  settings.use_mineshafts,
				temples:     settings.use_temples,
				ravines:     settings.use_ravines
			},
			decorators: Decorators {
				dungeon_chance:    if settings.use_dungeons    { Some(settings.dungeon_chance)    } else { None },
				water_lake_chance: if settings.use_water_lakes { Some(settings.water_lake_chance) } else { None },
				lava_lake_chance:  if settings.use_lava_lakes  { Some(settings.lava_lake_chance)  } else { None },
				dirt: VeinSettings {
					size:  settings.dirt_size,
					count: settings.dirt_count,
					min_y: settings.dirt_min_height,
					max_y: settings.dirt_max_height
				},
				gravel: VeinSettings {
					size:  settings.gravel_size,
					count: settings.gravel_count,
					min_y: settings.gravel_min_height,
					max_y: settings.gravel_max_height
				},
				granite: VeinSettings {
					size:  settings.granite_size,
					count: settings.granite_count,
					min_y: settings.granite_min_height,
					max_y: settings.granite_max_height
				},
				diorite: VeinSettings {
					size:  settings.diorite_size,
					count: settings.diorite_count,
					min_y: settings.diorite_min_height,
					max_y: settings.diorite_max_height
				},
				andesite: VeinSettings {
					size:  settings.andesite_size,
					count: settings.andesite_count,
					min_y: settings.andesite_min_height,
					max_y: settings.andesite_max_height
				},
				coal: VeinSettings {
					size:  settings.coal_size,
					count: settings.coal_count,
					min_y: settings.coal_min_height,
					max_y: settings.coal_max_height
				},
				iron: VeinSettings {
					size:  settings.iron_size,
					count: settings.iron_count,
					min_y: settings.iron_min_height,
					max_y: settings.iron_max_height
				},
				redstone: VeinSettings {
					size:  settings.redstone_size,
					count: settings.redstone_count,
					min_y: settings.redstone_min_height,
					max_y: settings.redstone_max_height
				},
				diamond: VeinSettings {
					size:  settings.diamond_size,
					count: settings.diamond_count,
					min_y: settings.diamond_min_height,
					max_y: settings.diamond_max_height
				},
				lapis: VeinSettingsCentered {
					size:     settings.lapis_size,
					count:    settings.lapis_count,
					center_y: settings.lapis_center_height,
					spread:   settings.lapis_spread
				}
			}
		}
	}
}

#[derive(Debug, PartialEq)]
pub struct BiomeSettings {
	pub depth_weight: f64,
	pub depth_offset: f64,
	pub scale_weight: f64,
	pub scale_offset: f64,
	pub fixed:        i32,
	pub biome_size:   i32,
	pub river_size:   i32
}

#[derive(Debug, PartialEq)]
pub struct Ocean {
	pub top: i32,
	pub lava: bool
}

#[derive(Debug, PartialEq)]
pub struct Structures {
	pub caves:       bool,
	pub strongholds: bool,
	pub villages:    bool,
	pub mineshafts:  bool,
	pub temples:     bool,
	pub ravines:     bool
}

#[derive(Debug, PartialEq)]
pub struct Decorators {
	pub dungeon_chance:    Option<i32>,
	pub water_lake_chance: Option<i32>,
	pub lava_lake_chance:  Option<i32>,
	pub dirt:              VeinSettings,
	pub gravel:            VeinSettings,
	pub granite:           VeinSettings,
	pub diorite:           VeinSettings,
	pub andesite:          VeinSettings,
	pub coal:              VeinSettings,
	pub iron:              VeinSettings,
	pub redstone:          VeinSettings,
	pub diamond:           VeinSettings,
	pub lapis:             VeinSettingsCentered
}

#[derive(Debug, PartialEq)]
pub struct VeinSettings {
	pub size:  i32,
	pub count: i32,
	pub min_y: i32,
	pub max_y: i32
}

#[derive(Debug, PartialEq)]
pub struct VeinSettingsCentered {
	pub size:     i32,
	pub count:    i32,
	pub center_y: i32,
	pub spread:   i32
}