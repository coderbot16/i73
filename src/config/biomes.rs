use biome::{Grid, Biome, Surface, Followup};
use serde_json;
use distribution::{Chance, Baseline};
use std::collections::HashMap;
use std::num::ParseIntError;
use std::borrow::Cow;
use decorator::{Dispatcher, DecoratorFactory};

#[derive(Debug)]
pub enum Error {
	ParseInt(ParseIntError),
	UnknownBiome(String)
}

impl From<ParseIntError> for Error {
	fn from(from: ParseIntError) -> Self {
		Error::ParseInt(from)
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BiomesConfig {
	#[serde(default)]
	pub decorator_sets: HashMap<String, Vec<DecoratorConfig>>,
	pub biomes: HashMap<String, BiomeConfig>,
	pub default: String,
	pub grid: Vec<RectConfig>
}

impl BiomesConfig {
	pub fn to_grid(&self) -> Result<Grid<u16>, Error> {
		let mut translated = HashMap::with_capacity(self.biomes.capacity());
		
		for (name, biome) in &self.biomes {
			translated.insert(name.clone(), biome.to_biome()?);
		}
		
		let default = translated.get(&self.default).ok_or_else(|| Error::UnknownBiome(self.default.clone()))?;
		
		let mut grid = Grid::new(default.clone());
		
		for rect in &self.grid {
			let biome = translated.get(&rect.biome).ok_or_else(|| Error::UnknownBiome(rect.biome.clone()))?;
			
			grid.add(rect.temperature, rect.rainfall, biome.clone());
		}
		
		Ok(grid)
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BiomeConfig {
	pub debug_name: String,
	pub surface: SurfaceConfig,
	#[serde(default)]
	pub decorators: Vec<String>
}

impl BiomeConfig {
	pub fn to_biome(&self) -> Result<Biome<u16>, ParseIntError> {
		Ok(Biome {
			name: Cow::Owned(self.debug_name.clone()),
			surface: self.surface.to_surface()?
		})
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SurfaceConfig {
	pub top: String,
	pub fill: String,
	pub chain: Vec<FollowupConfig>
}

impl SurfaceConfig {
	pub fn to_surface(&self) -> Result<Surface<u16>, ParseIntError> {
		Ok(Surface {
			top: parse_id(&self.top)?,
			fill: parse_id(&self.fill)?,
			chain: self.chain.iter().map(FollowupConfig::to_followup).collect::<Result<Vec<Followup<u16>>, ParseIntError>>()?
		})
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FollowupConfig {
	pub block: String,
	pub max_depth: u32
}

impl FollowupConfig {
	pub fn to_followup(&self) -> Result<Followup<u16>, ParseIntError> {
		Ok(Followup {
			block: parse_id(&self.block)?,
			max_depth: self.max_depth
		})
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DecoratorConfig {
	pub decorator: String,
	pub settings: serde_json::Value,
	pub height_distribution: Chance<Baseline>,
	pub count: Chance<Baseline>
}

impl DecoratorConfig {
	pub fn into_dispatcher(self, registry: &HashMap<String, Box<DecoratorFactory<u16>>>) -> Result<Dispatcher<Chance<Baseline>, Chance<Baseline>, u16>, String> {
		let factory = registry.get(&self.decorator).ok_or_else(|| format!("unknown decorator kind: {}", self.decorator))?;

		let decorator =factory.configure(self.settings).map_err(|e| format!("{}", e))?;

		Ok(Dispatcher {
			decorator,
			height_distribution: self.height_distribution,
			rarity: self.count
		})
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RectConfig {
	pub temperature: (f64, f64),
	pub rainfall: (f64, f64),
	pub biome: String
}

pub fn parse_id(id: &str) -> Result<u16, ParseIntError> {
	let mut split = id.split(':');
	
	let primary = split.next().unwrap().parse::<u16>()?;
	let secondary = split.next().map(|s| s.parse::<u16>());
	
	let secondary = match secondary {
		Some(secondary) => secondary?,
		None => 0
	};
	
	Ok(primary * 16 + secondary)
}