use cgmath::Point2;
use noise::octaves::SimplexOctaves;
use java_rand::Random;
use sample::Sample;

#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub struct ClimateSettings {
	pub temperature_fq:    f64,
	pub rainfall_fq:       f64,
	pub mixin_fq:          f64,
	pub temperature_mixin: f64,
	pub rainfall_mixin:    f64,
	pub temperature_mean:  f64,
	pub temperature_coeff: f64,
	pub rainfall_mean:     f64,
	pub rainfall_coeff:    f64,
	pub mixin_mean:        f64,
	pub mixin_coeff:       f64
}

impl Default for ClimateSettings {
	fn default() -> Self {
		ClimateSettings {
			temperature_fq:    0.25,
			rainfall_fq:       1.0/3.0,
			mixin_fq:          1.0/1.7,
			temperature_mixin: 0.010,
			rainfall_mixin:    0.002,
			temperature_mean:  0.7,
			temperature_coeff: 0.15,
			rainfall_mean:     0.5,
			rainfall_coeff:    0.15,
			mixin_mean:        0.5,
			mixin_coeff:       1.1
		}
	}
}

const  TEMP_COEFF: i64 = 9871;
const  RAIN_COEFF: i64 = 39811;
const MIXIN_COEFF: i64 = 543321;

#[derive(Debug)]
pub struct ClimateSource {
	temperature: SimplexOctaves,
	rainfall:    SimplexOctaves,
	mixin:       SimplexOctaves,
	settings:    ClimateSettings,
	temp_keep:   f64,
	rain_keep:   f64
}

impl ClimateSource {
	pub fn new(seed: u64, settings: ClimateSettings) -> Self {
		let seed = seed as i64;
		let scale = (1 << 4) as f64;
		
		ClimateSource {
			temperature: SimplexOctaves::new(&mut Random::new(seed.wrapping_mul(TEMP_COEFF) as u64),  4, settings.temperature_fq, 0.5, (0.4 / scale, 0.4 / scale)),
			rainfall:    SimplexOctaves::new(&mut Random::new(seed.wrapping_mul(RAIN_COEFF) as u64),  4, settings.rainfall_fq,    0.5, (0.8 / scale, 0.8 / scale)),
			mixin:       SimplexOctaves::new(&mut Random::new(seed.wrapping_mul(MIXIN_COEFF) as u64), 2, settings.mixin_fq,       0.5, (4.0 / scale, 4.0 / scale)),
			settings,
			temp_keep:   1.0 - settings.temperature_mixin,
			rain_keep:   1.0 - settings.rainfall_mixin
		}
	}
}

impl Sample for ClimateSource {
	type Output = Climate;
	
	fn sample(&self, point: Point2<f64>) -> Self::Output {
		let mixin = self.mixin.sample(point) * self.settings.mixin_coeff + self.settings.mixin_mean;
		
		let temp = (self.temperature.sample(point) * self.settings.temperature_coeff + self.settings.temperature_mean) * self.temp_keep + mixin * self.settings.temperature_mixin;
		let rain =    (self.rainfall.sample(point) * self.settings.rainfall_coeff    + self.settings.rainfall_mean   ) * self.rain_keep + mixin * self.settings.rainfall_mixin;
		
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
	/// Returns a Climate that represents Minecraft Alpha terrain.
	pub fn alpha() -> Self {
		Climate {
			temperature: 1.0,
			rainfall:    1.0
		}
	}
	
	pub fn new(temperature: f64, rainfall: f64) -> Self {
		Climate {
			temperature: temperature.max(0.0).min(1.0),
			rainfall:       rainfall.max(0.0).min(1.0)
		}
	}
	
	pub fn freezing(&self) -> bool {
		self.temperature < 0.5
	}
	
	pub fn temperature(&self) -> f64 {
		self.temperature
	}
	
	pub fn rainfall(&self) -> f64 {
		self.rainfall
	}
	
	pub fn adjusted_rainfall(&self) -> f64 {
		self.temperature * self.rainfall
	}
	
	/// Returns a value between 0.0 and 1.0 that lowers/raises the chaos.
	/// Temperature and Rainfall at 100% results in 1.0, which is the 
	/// influence factor for generators without biomes.
	/// This means that no biome is in fact signalling rainforest-like terrain.
	pub fn influence_factor(&self) -> f64 {
		1.0 - f64::powi(1.0 - self.adjusted_rainfall(), 4)
	}
}