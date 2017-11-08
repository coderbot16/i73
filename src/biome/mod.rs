pub mod climate;
pub mod storage;
pub mod source;

use biome::climate::Climate;
use chunk::storage::Target;
use std::borrow::Cow;
use segmented::Segmented;

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub struct Biome<B> where B: Target {
	pub surface: Surface<B>,
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

#[derive(Debug)]
pub struct Grid<B>(pub Segmented<Segmented<Biome<B>>>) where B: Target;
impl<B> Grid<B> where B: Target {
	fn new_temperatures(biome: Biome<B>) -> Segmented<Biome<B>> {
		let mut temperatures = Segmented::new(biome.clone());
		temperatures.add_boundary(1.0, biome.clone());
		
		temperatures
	}
	
	pub fn new(default: Biome<B>) -> Self {
		let temperatures = Self::new_temperatures(default);
		
		let mut grid = Segmented::new(temperatures.clone());
		grid.add_boundary(1.0, temperatures.clone());
		
		Grid(grid)
	}
	
	pub fn add(&mut self, temperature: (f64, f64), rainfall: (f64, f64), biome: Biome<B>) {
		self.0.for_all_aligned(rainfall.0, rainfall.1, &|| Self::new_temperatures(biome.clone()), &|temperatures| {
			temperatures.for_all_aligned(temperature.0, temperature.1, &|| biome.clone(), &|existing| {
				*existing = biome.clone();
			})
		})
	}
	
	pub fn lookup(&self, climate: Climate) -> &Biome<B> {
		self.0.get(climate.adjusted_rainfall()).get(climate.temperature())
	}
}

pub struct Lookup<B>(Box<[Biome<B>]>) where B: Target;
impl<B> Lookup<B> where B: Target {
	pub fn filled(biome: &Biome<B>) -> Self {
		let mut lookup = Vec::with_capacity(4096);
		
		for _ in 0..4096 {
			lookup.push(biome.clone());
		}
		
		Lookup(lookup.into_boxed_slice())
	}
	
	pub fn generate(grid: &Grid<B>) -> Self {
		let mut lookup = Vec::with_capacity(4096);
		
		for index in 0..4096 {
			let (temperature, rainfall) = (index / 64, index % 64);
			
			let climate = Climate::new((temperature as f64) / 63.0, (rainfall as f64) / 63.0);
				
			lookup.push(grid.lookup(climate).clone());
		}
		
		Lookup(lookup.into_boxed_slice())
	}
	
	pub fn lookup_raw(&self, temperature: usize, rainfall: usize) -> &Biome<B> {
		&self.0[temperature * 64 + rainfall]
	}
	
	pub fn lookup(&self, climate: Climate) -> &Biome<B> {
		self.lookup_raw((climate.temperature() * 63.0) as usize, (climate.rainfall() * 63.0) as usize)
	}
}