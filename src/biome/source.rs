use biome::storage::Layer;
use chunk::position::LayerPosition;
use biome::{Biome, Lookup};
use biome::climate::ClimateSource;
use chunk::storage::Target;
use nalgebra::Vector2;
use sample::Sample;

pub struct BiomeSource<B> where B: Target {
	climate: ClimateSource,
	lookup:  Lookup<B>
}

impl<B> BiomeSource<B> where B: Target {
	pub fn new(seed: i64, lookup: Lookup<B>) -> Self {
		BiomeSource {
			climate: ClimateSource::new(seed),
			lookup
		}
	}
	
	pub fn layer(&self, chunk: (i32, i32)) -> Layer<Biome<B>> {
		let block = (
			(chunk.0 * 16) as f64,
			(chunk.1 * 16) as f64
		);
		
		let mut layer = Layer::new(2);
		
		for z in 0..16 {
			for x in 0..16 {
				let position = LayerPosition::new(x, z);
				
				let climate = self.climate.sample(Vector2::new(block.0 + x as f64, block.1 + z as f64));
				let biome = self.lookup.lookup(climate);
				
				layer.set_immediate(position, biome);
			}
		}
		
		layer
	}
}