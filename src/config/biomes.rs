use biome::{Grid, Biome, Surface, Followup};
use std::collections::HashMap;
use std::num::ParseIntError;
use std::borrow::Cow;

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
	pub surface: SurfaceConfig
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