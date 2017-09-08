use noise::octaves::PerlinOctaves;
use nalgebra::Vector3;

struct TriNoiseSettings {
	main_scale:        f64,
	upper_limit_scale: f64,
	lower_limit_scale: f64
}

impl Default for TriNoiseSettings {
	fn default() -> Self {
		TriNoiseSettings {
			main_scale:        20.0,
			upper_limit_scale: 512.0,
			lower_limit_scale: 512.0
		}
	}
}

struct TriNoiseSource {
	lower: PerlinOctaves,
	upper: PerlinOctaves,
	main:  PerlinOctaves,
	main_scale:        f64,
	upper_limit_scale: f64,
	lower_limit_scale: f64
}

impl TriNoiseSource {
	fn sample(&self, point: Vector3<f64>, index: usize) -> f64 {
		let lower = self.lower.generate_override(point, index) / self.lower_limit_scale;
		let upper = self.upper.generate_override(point, index) / self.upper_limit_scale;
		let main  =  self.main.generate_override(point, index) / self.main_scale + 0.5;
		
		lerp(main.max(0.0).min(1.0), lower, upper)
	}
}

/*
	main_noise_scale: (f64, f64, f64),
	depth_noise_scale: (f64, f64),
	depth_base: f64,
	coordinate_scale: f64,
	height_scale: f64,
*/

struct FieldSettings {
	ground_stretch :   f64,
	seabed_stretch:    f64,
	taper_threshold:   f64,
	height_stretch:    f64
}

impl Default for FieldSettings {
	fn default() -> Self {
		FieldSettings {
			ground_stretch:    4.0,
			seabed_stretch:    1.0,
			taper_threshold:   13.0,
			height_stretch:    12.0
		}
	}
}

impl FieldSettings {
	// TODO: Replace with FieldSource.
	fn compute_noise_value(&self, y: f64, height_center: f64, chaos: f64, tri_noise: f64) -> f64 {
		// Reduction factor is 0 if y <= Y_THRESH.
		let reduction_factor = (y.max(self.taper_threshold) - self.taper_threshold) / 3.0;
		let distance = y - height_center;
		let distance = distance * if distance < 0.0 {self.ground_stretch} else {self.seabed_stretch};
		
		let reduction = distance * self.height_stretch / chaos;
		let value = tri_noise - reduction;
		
		value * (1.0 - reduction_factor) - 10.0*reduction_factor
	}
}

fn lerp(t: f64, a: f64, b: f64) -> f64 {
	a + t * (b - a)
}