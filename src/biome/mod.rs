pub mod climate;
pub mod storage;
pub mod source;

use biome::climate::Climate;
use chunk::storage::Target;
use std::borrow::Cow;
use std::fmt::Display;
use segmented::Segmented;

pub fn default_grid() -> Grid<u16, char> {
	let sandstone = Followup {
		block:     24*16,
		max_depth: 3
	};
	
	//           Biome { surface: Surface {top: 12*16, fill: 12*16, chain: vec![sandstone.clone()] }, id: '9', name: Cow::Borrowed("Ice Desert"     ) };
	let plains = Biome { surface: Surface {top: 35*16 + 8, fill:  3*16, chain: vec![                 ] }, id: '8', name: Cow::Borrowed("Plains"         ) };
	let tundra = Biome { surface: Surface {top: 35*16 + 1, fill:  3*16, chain: vec![                 ] }, id: 'A', name: Cow::Borrowed("Tundra"         ) };
	let forest = Biome { surface: Surface {top: 35*16 + 6, fill:  3*16, chain: vec![                 ] }, id: '3', name: Cow::Borrowed("Forest"         ) };
	
	let mut grid = Grid::new(plains.clone());
	
	grid.add((0.00, 0.10), (0.00, 1.00), tundra.clone()                                                                                                                        );
	grid.add((0.10, 0.50), (0.00, 0.20), tundra                                                                                                                                );
	grid.add((0.10, 0.50), (0.20, 0.50), Biome { surface: Surface {top: 35*16 + 2, fill:  3*16, chain: vec![                 ] }, id: '6', name: Cow::Borrowed("Taiga"          ) });
	grid.add((0.10, 0.70), (0.50, 1.00), Biome { surface: Surface {top: 35*16 + 3, fill:  3*16, chain: vec![                 ] }, id: '1', name: Cow::Borrowed("Swampland"      ) });
	grid.add((0.50, 0.95), (0.00, 0.20), Biome { surface: Surface {top: 2*16/*35*16 + 4*/, fill:  3*16, chain: vec![                 ] }, id: '4', name: Cow::Borrowed("Savanna"        ) });
	grid.add((0.50, 0.97), (0.20, 0.35), Biome { surface: Surface {top: 35*16 + 5, fill:  3*16, chain: vec![                 ] }, id: '5', name: Cow::Borrowed("Shrubland"      ) });
	grid.add((0.50, 0.97), (0.35, 0.50), forest.clone()                                                                                                                        );
	grid.add((0.70, 0.97), (0.50, 1.00), forest                                                                                                                                );
	grid.add((0.95, 1.00), (0.00, 0.20), Biome { surface: Surface {top: 35*16 + 7, fill: 12*16, chain: vec![sandstone        ] }, id: '7', name: Cow::Borrowed("Desert"         ) });
	grid.add((0.97, 1.00), (0.20, 0.45), plains                                                                                                                                );
	grid.add((0.97, 1.00), (0.45, 0.90), Biome { surface: Surface {top: 35*16 + 9, fill:  3*16, chain: vec![                 ] }, id: '2', name: Cow::Borrowed("Seasonal Forest") });
	grid.add((0.97, 1.00), (0.90, 1.00), Biome { surface: Surface {top: 35*16 + 10, fill:  3*16, chain: vec![                 ] }, id: '0', name: Cow::Borrowed("Rainforest"     ) });
	
	grid
}

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub struct Biome<B, I> where B: Target, I: Clone {
	pub surface: Surface<B>,
	pub id:   I,
	pub name: Cow<'static, str>
}

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub struct Surface<B> where B: Target {
	pub top:  B,
	pub fill: B,
	pub chain: Vec<Followup<B>>
}

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub struct Followup<B> where B: Target {
	pub block:     B,
	pub max_depth: u32
}

pub struct Grid<B, I>(pub Segmented<Segmented<Biome<B, I>>>) where B: Target, I: Clone;
impl<B, I> Grid<B, I> where B: Target, I: Clone {
	fn new_temperatures(biome: Biome<B, I>) -> Segmented<Biome<B, I>> {
		let mut temperatures = Segmented::new(biome.clone());
		temperatures.add_boundary(1.0, biome.clone());
		
		temperatures
	}
	
	pub fn new(default: Biome<B, I>) -> Self {
		let temperatures = Self::new_temperatures(default);
		
		let mut grid = Segmented::new(temperatures.clone());
		grid.add_boundary(1.0, temperatures.clone());
		
		Grid(grid)
	}
	
	pub fn add(&mut self, temperature: (f64, f64), rainfall: (f64, f64), biome: Biome<B, I>) {
		self.0.for_all_aligned(rainfall.0, rainfall.1, &|| Self::new_temperatures(biome.clone()), &|temperatures| {
			temperatures.for_all_aligned(temperature.0, temperature.1, &|| biome.clone(), &|existing| {
				*existing = biome.clone();
			})
		})
	}
	
	pub fn lookup(&self, climate: Climate) -> &Biome<B, I> {
		self.0.get(climate.adjusted_rainfall()).get(climate.temperature())
	}
}

pub struct Lookup<B, I>(Box<[Biome<B, I>]>) where B: Target, I: Clone;
impl<B, I> Lookup<B, I> where B: Target, I: Clone {
	pub fn filled(biome: &Biome<B, I>) -> Self {
		let mut lookup = Vec::with_capacity(4096);
		
		for index in 0..4096 {
			lookup.push(biome.clone());
		}
		
		Lookup(lookup.into_boxed_slice())
	}
	
	pub fn generate(grid: &Grid<B, I>) -> Self {
		let mut lookup = Vec::with_capacity(4096);
		
		for index in 0..4096 {
			let (temperature, rainfall) = (index / 64, index % 64);
			
			let climate = Climate::new((temperature as f64) / 63.0, (rainfall as f64) / 63.0);
				
			lookup.push(grid.lookup(climate).clone());
		}
		
		Lookup(lookup.into_boxed_slice())
	}
	
	fn lookup_raw(&self, temperature: usize, rainfall: usize) -> &Biome<B, I> {
		&self.0[temperature * 64 + rainfall]
	}
	
	pub fn lookup(&self, climate: Climate) -> &Biome<B, I> {
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