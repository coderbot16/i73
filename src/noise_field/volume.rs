use rng::JavaRng;
use noise::octaves::PerlinOctavesVertical;
use nalgebra::Vector3;
use noise_field::height::Height;

pub const H_NOISE_SIZE: usize = 5;
pub const Y_NOISE_SIZE: usize = 17;

#[derive(Debug)]
pub struct TriNoiseSettings {
	pub  main_out_scale: f64,
	pub upper_out_scale: f64,
	pub lower_out_scale: f64,
	pub lower_scale:     Vector3<f64>,
	pub upper_scale:     Vector3<f64>,
	pub  main_scale:     Vector3<f64>,
}

impl Default for TriNoiseSettings {
	fn default() -> Self {
		TriNoiseSettings {
			 main_out_scale:  20.0,
			upper_out_scale: 512.0,
			lower_out_scale: 512.0,
			lower_scale:     Vector3::new(684.412,        684.412,         684.412       ),
			upper_scale:     Vector3::new(684.412,        684.412,         684.412       ),
			 main_scale:     Vector3::new(684.412 / 80.0, 684.412 / 160.0, 684.412 / 80.0)
		}
	}
}

pub struct TriNoiseSource {
	lower:           PerlinOctavesVertical,
	upper:           PerlinOctavesVertical,
	main:            PerlinOctavesVertical,
	 main_out_scale: f64,
	upper_out_scale: f64,
	lower_out_scale: f64
}

impl TriNoiseSource {
	pub fn new(rng: &mut JavaRng, settings: &TriNoiseSettings) -> Self { 
		TriNoiseSource {
			lower: PerlinOctavesVertical::new(rng, 16, settings.lower_scale, 0.0, Y_NOISE_SIZE),
			upper: PerlinOctavesVertical::new(rng, 16, settings.upper_scale, 0.0, Y_NOISE_SIZE),
			 main: PerlinOctavesVertical::new(rng,  8, settings. main_scale, 0.0, Y_NOISE_SIZE),
			 main_out_scale: settings. main_out_scale,
			upper_out_scale: settings.upper_out_scale,
			lower_out_scale: settings.lower_out_scale
		}
	}
	
	pub fn sample(&self, point: Vector3<f64>, index: usize) -> f64 {
		let lower = self.lower.generate_override(point, index) / self.lower_out_scale;
		let upper = self.upper.generate_override(point, index) / self.upper_out_scale;
		let main  = self. main.generate_override(point, index) / self. main_out_scale + 0.5;
		
		lerp(main.max(0.0).min(1.0), lower, upper)
	}
}

#[derive(Debug)]
pub struct FieldSettings {
	pub seabed_stretch :   f64,
	pub ground_stretch:    f64,
	pub taper_threshold:   f64,
	pub height_stretch:    f64
}

impl Default for FieldSettings {
	fn default() -> Self {
		FieldSettings {
			seabed_stretch:    4.0,
			ground_stretch:    1.0,
			taper_threshold:   13.0,
			height_stretch:    12.0
		}
	}
}

impl FieldSettings {
	// TODO: Replace with FieldSource.
	pub fn compute_noise_value(&self, y: f64, height: Height, tri_noise: f64) -> f64 {
		// Reduction factor is 0 if y <= taper_threshold.
		let reduction_factor = (y.max(self.taper_threshold) - self.taper_threshold) / 3.0;
		let distance = y - height.center;
		let distance = distance * if distance < 0.0 {self.seabed_stretch} else {self.ground_stretch};
		
		let reduction = distance * self.height_stretch / height.chaos;
		let value = tri_noise - reduction;
		
		value * (1.0 - reduction_factor) - 10.0*reduction_factor
	}
}

fn lerp(t: f64, a: f64, b: f64) -> f64 {
	a + t * (b - a)
}