use surface::Surface;
use climate::Climate;
use std::ops::Range;

#[derive(Debug, Copy, Clone)]
pub enum Biome {
	Rainforest,
	Swampland,
	SeasonalForest,
	Forest,
	Savanna,
	Shrubland,
	Taiga,
	Desert,
	Plains,
	//IceDesert,
	Tundra
}

impl Biome {
	pub fn surface(&self) -> () {
		/*match *self {
			Biome::Desert => Surface { top: Some(Block::Sand ), fill: Block::Sand },
			_			  => Surface { top: Some(Block::Grass), fill: Block::Dirt }
		}*/
		
		// TODO: Properly replace this with the AnvilId system.
		// Also replace biomes with structs instead.
		
		unimplemented!()
	}
	
	pub fn shorthand(&self) -> char {
		match *self {
			Biome::Rainforest     => 'R',
			Biome::Swampland      => 'S',
			Biome::SeasonalForest => 'G',
			Biome::Forest         => 'F',
			Biome::Savanna        => 'Q',
			Biome::Shrubland      => 'N',
			Biome::Taiga          => 'T',
			Biome::Desert         => 'D',
			Biome::Plains         => 'P',
			Biome::Tundra         => 'U'
		}
	}
}

// TODO: Add Grid to allow for configurable biomes, and a possibly? faster lookup.

/*pub struct Grid(Vec<RainColumn>);
impl Grid {
	pub fn new() -> Self {
		Grid(Vec::new())
	}
	
	pub fn insert(&mut self, temperature: Range<f64>, rainfall: Range<f64>, biome: Biome) {
		unimplemented!()
		// This function needs to locate the overlapping columns, subdivide them if neccesary, and for each overlapping column do the same thing with the rows inside.
	}
	
	fn find_column(&self, rainfall: f64) -> Option<&[Cell]> {
		for &RainColumn { ref rain, ref entries } in &self.0 {
			if rain.contains(rainfall) {
				return Some(&entries);
			}
		}
		
		None
	}
	
	fn find_column_mut(&mut self, rainfall: f64) -> Option<&mut Vec<Cell>> {
		for &mut RainColumn { ref rain, ref mut entries } in &mut self.0 {
			if rain.contains(rainfall) {
				return Some(entries);
			}
		}
		
		None
	}
	
	pub fn lookup(&self, climate: Climate) -> Option<Biome> {
		match self.find_column(climate.rainfall()) {
			Some(entries) => {
				for &Cell { ref temperature, biome } in entries {
					if temperature.contains(climate.temperature()) {
						return Some(biome);
					}
				}
				
				None
			},
			None => None
		}
	}
}*/

/*struct Grid(Vec<RainColumn>);

impl Grid {
	
}

pub struct Selection<T> {
	range: Range<f64>,
	part: T
}

type RainColumn = Selection<Vec<TempRow>>;
type TempRow = Selection<Biome>;*/

pub struct Lookup(pub [[Biome; 64]; 64]);
impl Lookup {
	pub fn generate(/*grid: &Grid*/) -> Option<Self> {
		let mut lookup = [[Biome::Plains; 64]; 64];
		
		for (temperature, rainfalls) in lookup.iter_mut().enumerate() {
			for (rainfall, biome) in rainfalls.iter_mut().enumerate() {
				let climate = Climate::new((temperature as f64) / 63.0, (rainfall as f64) / 63.0);
				
				*biome = climate.biome_exact();
				
				/*match grid.lookup(climate) {
					Some(cell) => *biome = cell,
					None => return None
				}*/
			}
		}
		
		Some(Lookup(lookup))
	}
	
	pub fn lookup(&self, climate: Climate) -> Biome {
		let (temperature, rainfall) = ((climate.temperature() * 63.0) as usize, (climate.rainfall() * 63.0) as usize);
		self.0[temperature][rainfall]
	}
}

impl ::std::fmt::Display for Lookup {
	fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
		for rain in 0..64 {
			for temp in 0..64 {
				write!(f, "{} ", self.0[temp][63-rain].shorthand())?;
			}
			writeln!(f, "")?;
		}
		
		Ok(())
	}
}