use surface::Surface;
use climate::Climate;
use chunk::storage::Target;
use std::borrow::Cow;
use std::fmt::Display;
use segmented::Segmented;

pub static DEFAULT_BIOMES: [BiomeDef<u16, char>; 11] = [
	BiomeDef { surface: Surface {top:  2, fill:  3 }, id: '0', name: Cow::Borrowed("Rainforest"     ) },
	BiomeDef { surface: Surface {top:  2, fill:  3 }, id: '1', name: Cow::Borrowed("Swampland"      ) },
	BiomeDef { surface: Surface {top:  2, fill:  3 }, id: '2', name: Cow::Borrowed("Seasonal Forest") },
	BiomeDef { surface: Surface {top:  2, fill:  3 }, id: '3', name: Cow::Borrowed("Forest"         ) },
	BiomeDef { surface: Surface {top:  2, fill:  3 }, id: '4', name: Cow::Borrowed("Savanna"        ) },
	BiomeDef { surface: Surface {top:  2, fill:  3 }, id: '5', name: Cow::Borrowed("Shrubland"      ) },
	BiomeDef { surface: Surface {top:  2, fill:  3 }, id: '6', name: Cow::Borrowed("Taiga"          ) },
	BiomeDef { surface: Surface {top: 12, fill: 12 }, id: '7', name: Cow::Borrowed("Desert"         ) },
	BiomeDef { surface: Surface {top:  2, fill:  3 }, id: '8', name: Cow::Borrowed("Plains"         ) },
	BiomeDef { surface: Surface {top: 12, fill: 12 }, id: '9', name: Cow::Borrowed("Ice Desert"     ) },
	BiomeDef { surface: Surface {top:  2, fill:  3 }, id: 'A', name: Cow::Borrowed("Tundra"         ) }
];

pub fn default_grid() -> Grid<u16, char> {
	let mut grid = Grid::new(DEFAULT_BIOMES[0].clone());
	
	grid.add((0.00, 0.10), (0.00, 1.00), DEFAULT_BIOMES[Biome::Tundra as usize].clone());
	grid.add((0.10, 0.50), (0.00, 0.20), DEFAULT_BIOMES[Biome::Tundra as usize].clone());
	grid.add((0.10, 0.50), (0.20, 0.50), DEFAULT_BIOMES[Biome::Taiga as usize].clone());
	grid.add((0.10, 0.70), (0.50, 1.00), DEFAULT_BIOMES[Biome::Swampland as usize].clone());
	grid.add((0.50, 0.95), (0.00, 0.20), DEFAULT_BIOMES[Biome::Savanna as usize].clone());
	grid.add((0.50, 0.97), (0.20, 0.35), DEFAULT_BIOMES[Biome::Shrubland as usize].clone());
	grid.add((0.50, 0.97), (0.35, 0.50), DEFAULT_BIOMES[Biome::Forest as usize].clone());
	grid.add((0.70, 0.97), (0.50, 1.00), DEFAULT_BIOMES[Biome::Forest as usize].clone());
	grid.add((0.95, 1.00), (0.00, 0.20), DEFAULT_BIOMES[Biome::Desert as usize].clone());
	grid.add((0.97, 1.00), (0.20, 0.45), DEFAULT_BIOMES[Biome::Plains as usize].clone());
	grid.add((0.97, 1.00), (0.45, 0.90), DEFAULT_BIOMES[Biome::SeasonalForest as usize].clone());
	grid.add((0.97, 1.00), (0.90, 1.00), DEFAULT_BIOMES[Biome::Rainforest as usize].clone());
	
	grid
}

#[derive(Debug, Clone)]
pub struct BiomeDef<B, I> where B: Target, I: Clone {
	pub surface: Surface<B>,
	pub id:   I,
	pub name: Cow<'static, str>
}

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
	IceDesert,
	Tundra
}

pub struct Grid<B, I>(pub Segmented<Segmented<BiomeDef<B, I>>>) where B: Target, I: Clone;
impl<B, I> Grid<B, I> where B: Target, I: Clone {
	fn new_temperatures(biome: BiomeDef<B, I>) -> Segmented<BiomeDef<B, I>> {
		let mut temperatures = Segmented::new(biome.clone());
		temperatures.add_boundary(1.0, biome.clone());
		
		temperatures
	}
	
	pub fn new(default: BiomeDef<B, I>) -> Self {
		let temperatures = Self::new_temperatures(default);
		
		let mut grid = Segmented::new(temperatures.clone());
		grid.add_boundary(1.0, temperatures.clone());
		
		Grid(grid)
	}
	
	pub fn add(&mut self, temperature: (f64, f64), rainfall: (f64, f64), biome: BiomeDef<B, I>) {
		self.0.for_all_aligned(rainfall.0, rainfall.1, &|| Self::new_temperatures(biome.clone()), &|temperatures| {
			temperatures.for_all_aligned(temperature.0, temperature.1, &|| biome.clone(), &|existing| {
				*existing = biome.clone();
			})
		})
	}
	
	pub fn lookup(&self, climate: Climate) -> &BiomeDef<B, I> {
		self.0.get(climate.adjusted_rainfall()).get(climate.temperature())
	}
}

pub struct Lookup<B, I>(Box<[BiomeDef<B, I>]>) where B: Target, I: Clone;
impl<B, I> Lookup<B, I> where B: Target, I: Clone {
	pub fn generate(grid: &Grid<B, I>) -> Self {
		let mut lookup = Vec::with_capacity(4096);
		
		for index in 0..4096 {
			let (temperature, rainfall) = (index / 64, index % 64);
			
			let climate = Climate::new((temperature as f64) / 63.0, (rainfall as f64) / 63.0);
				
			lookup.push(grid.lookup(climate).clone());
		}
		
		Lookup(lookup.into_boxed_slice())
	}
	
	fn lookup_raw(&self, temperature: usize, rainfall: usize) -> &BiomeDef<B, I> {
		&self.0[temperature * 64 + rainfall]
	}
	
	pub fn lookup(&self, climate: Climate) -> &BiomeDef<B, I> {
		self.lookup_raw((climate.temperature() * 63.0) as usize, (climate.rainfall() * 63.0) as usize)
	}
}

impl<B, I> Display for Lookup<B, I> where B: Target, I: Clone + Display {
	fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
		for rain in 0..64 {
			for temp in 0..64 {
				write!(f, "{} ", self.lookup_raw(temp, 63-rain).id)?;
			}
			writeln!(f, "")?;
		}
		
		Ok(())
	}
}


/*
	/// Gets the exact biome corresponding to the climate. Prefer a biome::Lookup instead.
	pub fn biome_exact(&self) -> Biome {
		match (self.temperature, self.adjusted_rainfall()) {
			(0.00.. 0.10,	0.00...1.00) => Biome::Tundra,
			(0.10.. 0.50,  	0.00.. 0.20) => Biome::Tundra,
			(0.10.. 0.50,	0.20.. 0.50) => Biome::Taiga,
			(0.10.. 0.70,	0.50...1.00) => Biome::Swampland,
			(0.50.. 0.95,	0.00.. 0.20) => Biome::Savanna,
			(0.50.. 0.97,	0.20.. 0.35) => Biome::Shrubland,
			(0.50.. 0.97,  	0.35.. 0.50) => Biome::Forest,
			(0.70.. 0.97,	0.50...1.00) => Biome::Forest,
			(0.95...1.00,	0.00.. 0.20) => Biome::Desert,
			(0.97...1.00,	0.20.. 0.45) => Biome::Plains,
			(0.97...1.00,	0.45.. 0.90) => Biome::SeasonalForest,
			(0.97...1.00,	0.90...1.00) => Biome::Rainforest,
			(_,_) => unreachable!()
		}
	}
*/