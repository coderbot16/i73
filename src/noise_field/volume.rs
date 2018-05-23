use java_rand::Random;
use noise::octaves::PerlinOctavesVertical;
use nalgebra::Vector3;
use noise_field::height::Height;
use vocs::position::ColumnPosition;

#[derive(Debug, PartialEq)]
pub struct TriNoiseSettings {
	pub  main_out_scale: f64,
	pub upper_out_scale: f64,
	pub lower_out_scale: f64,
	pub lower_scale:     Vector3<f64>,
	pub upper_scale:     Vector3<f64>,
	pub  main_scale:     Vector3<f64>,
	pub y_size:          usize
}

impl Default for TriNoiseSettings {
	fn default() -> Self {
		TriNoiseSettings {
			 main_out_scale:  20.0,
			upper_out_scale: 512.0,
			lower_out_scale: 512.0,
			lower_scale:     Vector3::new(684.412,        684.412,         684.412       ),
			upper_scale:     Vector3::new(684.412,        684.412,         684.412       ),
			 main_scale:     Vector3::new(684.412 / 80.0, 684.412 / 160.0, 684.412 / 80.0),
			y_size:          17
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
	pub fn new(rng: &mut Random, settings: &TriNoiseSettings) -> Self {
		TriNoiseSource {
			lower: PerlinOctavesVertical::new(rng, 16, settings.lower_scale, 0.0, settings.y_size),
			upper: PerlinOctavesVertical::new(rng, 16, settings.upper_scale, 0.0, settings.y_size),
			 main: PerlinOctavesVertical::new(rng,  8, settings. main_scale, 0.0, settings.y_size),
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
	pub taper_control:     f64,
	pub height_stretch:    f64
}

impl FieldSettings {
	pub fn with_height_stretch(height_stretch: f64) -> Self {
		let mut default = Self::default();
		
		default.height_stretch = height_stretch;
		
		default
	}
}

impl Default for FieldSettings {
	fn default() -> Self {
		FieldSettings {
			seabed_stretch:    4.0,
			ground_stretch:    1.0,
			taper_control:     4.0,
			height_stretch:    12.0
		}
	}
}

impl FieldSettings {
	// TODO: Replace with FieldSource.
	pub fn compute_noise_value(&self, y: f64, height: Height, tri_noise: f64) -> f64 {
		let distance = y - height.center;
		let distance = distance * if distance < 0.0 {self.seabed_stretch} else {self.ground_stretch};
		
		let reduction = distance * self.height_stretch / height.chaos;
		let value = tri_noise - reduction;
		
		reduce_upper(value, y, self.taper_control, 10.0, 17.0)
	}
}

fn lerp(t: f64, a: f64, b: f64) -> f64 {
	a + t * (b - a)
}

pub fn reduce_upper(value: f64, y: f64, control: f64, min: f64, max_y: f64) -> f64 {
	let threshold = max_y - control;
	let divisor   = control - 1.0;
	let factor    = (y.max(threshold) - threshold) / divisor;
	
	reduce(value, factor, min)
}

pub fn reduce_lower(value: f64, y: f64, control: f64, min: f64) -> f64 {
	let divisor   = control - 1.0;
	let factor    = (control - y.min(control)) / divisor;
	
	reduce(value, factor, min)
}

pub fn reduce_cubic(value: f64, distance: f64) -> f64 {
	let factor = 4.0 - distance.min(4.0);
	value - 10.0 * factor.powi(3)
}

pub fn reduce(value: f64, factor: f64, min: f64) -> f64 {
	value * (1.0 - factor) - min * factor
}

pub fn trilinear128(array: &[[[f64; 5]; 17]; 5], position: ColumnPosition) -> f64 {
	debug_assert!(position.y() < 128, "trilinear128 only supports Y values below 128");

	let inner = (
		((position.x() % 4) as f64) / 4.0,
		((position.y() % 8) as f64) / 8.0,
		((position.z() % 4) as f64) / 4.0
	);
	
	let indices = (
		(position.x() / 4) as usize,
		(position.y() / 8) as usize,
		(position.z() / 4) as usize
	);
	
	lerp(inner.2, 
		lerp(inner.0,
			lerp(inner.1,
				array[indices.0    ][indices.1    ][indices.2    ],
				array[indices.0    ][indices.1 + 1][indices.2    ],
			),
			lerp(inner.1,
				array[indices.0 + 1][indices.1    ][indices.2    ],
				array[indices.0 + 1][indices.1 + 1][indices.2    ],
			)
		),
		lerp(inner.0,
			lerp(inner.1,
				array[indices.0    ][indices.1    ][indices.2 + 1],
				array[indices.0    ][indices.1 + 1][indices.2 + 1],
			),
			lerp(inner.1,
				array[indices.0 + 1][indices.1    ][indices.2 + 1],
				array[indices.0 + 1][indices.1 + 1][indices.2 + 1],
			)
		)
	)
}