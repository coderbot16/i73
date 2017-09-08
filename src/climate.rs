use nalgebra::Vector2;
use noise::octaves::SimplexOctaves;
use rng::JavaRng;
use biome::Biome;
use sample::Sample;

const  TEMP_COEFF: i64 = 9871;
const  RAIN_COEFF: i64 = 39811;
const MIXIN_COEFF: i64 = 543321;

const     TEMP_FQ: f64 = 0.25;
const     RAIN_FQ: f64 = 1.0/3.0;
const    MIXIN_FQ: f64 = 1.0/1.7;

const  TEMP_MIXIN: f64 = 0.01;
const   TEMP_KEEP: f64 = 1.0 - TEMP_MIXIN;
const  RAIN_MIXIN: f64 = 0.002;
const   RAIN_KEEP: f64 = 1.0 - RAIN_MIXIN;

pub struct ClimateSource {
	temperature: SimplexOctaves,
	rainfall: SimplexOctaves,
	mixin: SimplexOctaves
}

impl ClimateSource {
	pub fn new(seed: i64) -> Self {
		ClimateSource {
			temperature: SimplexOctaves::new(&mut JavaRng::new(seed.wrapping_mul(TEMP_COEFF)),  4,  TEMP_FQ, 0.5, (0.025, 0.025)),
			rainfall:    SimplexOctaves::new(&mut JavaRng::new(seed.wrapping_mul(RAIN_COEFF)),  4,  RAIN_FQ, 0.5, (0.05,  0.05 )),
			mixin:       SimplexOctaves::new(&mut JavaRng::new(seed.wrapping_mul(MIXIN_COEFF)), 2, MIXIN_FQ, 0.5, (0.25,  0.25 )),
		}
	}
}

impl Sample for ClimateSource {
	type Output = Climate;
	
	fn sample(&self, point: Vector2<f64>) -> Self::Output {
		let mixin = self.mixin.sample(point) * 1.1 + 0.5;
		
		let temp = (self.temperature.sample(point) * 0.15 + 0.7) * TEMP_KEEP + mixin * TEMP_MIXIN;
		let rain =    (self.rainfall.sample(point) * 0.15 + 0.5) * RAIN_KEEP + mixin * RAIN_MIXIN;
		
		let temp = 1.0 - (1.0 - temp).powi(2);
		
		Climate::new(temp, rain)
	}
}

#[derive(Debug, Default, Copy, Clone)]
pub struct Climate {
	temperature: f64,
	rainfall: f64
}

impl Climate {
	pub fn new(temperature: f64, rainfall: f64) -> Self {
		Climate {
			temperature: temperature.max(0.0).min(1.0),
			rainfall:       rainfall.max(0.0).min(1.0)
		}
	}
	
	fn freezing(&self) -> bool {
		self.temperature < 0.5
	}
	
	pub fn temperature(&self) -> f64 {
		self.temperature
	}
	
	pub fn rainfall(&self) -> f64 {
		self.rainfall
	}
	
	fn adjusted_rainfall(&self) -> f64 {
		self.temperature * self.rainfall
	}
	
	/// scaled_noise is `noise / biome_influence_scale`
	pub fn chaos(&self, scaled_noise: f64) -> f64 {
		let factor = 1.0 - f64::powi(1.0 - self.adjusted_rainfall(), 4);
		let influence = factor * (scaled_noise + 0.5) + 0.5;
		influence.max(0.5).min(1.5)
	}
	
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
}