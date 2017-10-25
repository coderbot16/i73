use rng::NotchRng;
use image_ops::Image;
use image_ops::filter::Filter;

/// The Zoom filter resamples the input image to be 2 times larger on each axis.
pub struct Blur<T, S> where S: SelectBlur<T> {
	rng: NotchRng,
	selector: S,
	marker: ::std::marker::PhantomData<T>
}

impl<T, S> Blur<T, S> where S: SelectBlur<T> {
	pub fn new(rng: NotchRng, selector: S) -> Self {
		Blur {
			rng,
			selector,
			marker: ::std::marker::PhantomData
		}
	}
}

impl<T, S> Filter<T> for Blur<T, S> where S: SelectBlur<T>, T: Default + Copy + ::std::fmt::Debug {
	type Out = T;
	
	fn filter(&self, position: (i64, i64), image: &Image<T>, out: &mut Image<Self::Out>) {
		let sample_size = self.input_size((out.x_size(), out.z_size()));
		
		if image.x_size() < sample_size.0 {
			panic!("Source image X size of {} is less than the required minimum X size {} for blurring to a target X size of {}", image.x_size(), sample_size.0, out.x_size());
		}
		
		if image.z_size() < sample_size.1 {
			panic!("Source image Z size of {} is less than the required minimum Z size {} for blurring to a target Z size of {}", image.z_size(), sample_size.1, out.z_size());
		}
		
		for z in 0..out.z_size(){
			for x in 0..out.x_size() {
				let mut rng = self.rng.clone();
				rng.init_at(position.0 + x as i64, position.1 + z as i64);
				
				let values = (
					(
						*image.get(x,     z    ),
						*image.get(x,     z + 1),
						*image.get(x,     z + 2),
					),
					(
						*image.get(x + 1, z    ),
						*image.get(x + 1, z + 1),
						*image.get(x + 1, z + 2),
					),
					(
						*image.get(x + 2, z    ),
						*image.get(x + 2, z + 1),
						*image.get(x + 2, z + 2),
					)
				);
				
				out.set(x, z, self.selector.select(&mut rng, values));
			}
		}
	}
	
	fn input_size(&self, size: (usize, usize)) -> (usize, usize) {
		(
			size.0 + 2,
			size.1 + 2
		)
	}
	
	fn input_position(&self, position: (i64, i64)) -> (i64, i64) {
		(
			position.0 - 1,
			position.1 - 1
		)
	}
}

pub trait SelectBlur<T> {
	/// Area is a column-major matrix.
	fn select(&self, rng: &mut NotchRng, area: ((T, T, T), (T, T, T), (T, T, T))) -> T;
}

/// Causes diagonal neighbors to spill into the center. Named because of the sampling pattern - an "X" shape.
pub struct Denoise;

impl<T> SelectBlur<T> for Denoise where T: Eq {
	fn select(&self, rng: &mut NotchRng, area: ((T, T, T), (T, T, T), (T, T, T))) -> T {
		// Sampling pattern:
		//  X
		// XCX
		//  X
		
		match ((area.0).1 == (area.2).1, (area.1).0 == (area.1).2) {
			(true,  true ) => if rng.next_i32(2) == 0 { (area.0).1 } else { (area.1).0 },
			(true,  false) => (area.0).1,
			(false, true ) => (area.1).0,
			(false, false) => (area.1).1
		}
	}
}

/// Has a chance to remove lines and single pixel salt and pepper noise.
pub struct XSpill<T, M> where M: Mix<T> {
	mixer: M,
	marker: ::std::marker::PhantomData<T>
}

impl<T, M> XSpill<T, M> where M: Mix<T> {
	pub fn new(mixer: M) -> Self {
		XSpill {
			mixer,
			marker: ::std::marker::PhantomData
		}
	}
}

impl<T, M> SelectBlur<T> for XSpill<T, M> where T: Eq + ::std::fmt::Display, M: Mix<T> {
	fn select(&self, rng: &mut NotchRng, area: ((T, T, T), (T, T, T), (T, T, T))) -> T {
		// Sampling pattern:
		// X X
		//  C
		// X X
		
		let center = (area.1).1;
		
		let chance = self.mixer.spill_chance(&center);
		let can_spill = rng.next_i32(chance+1) == chance;
		
		let mut mask = ((center != (area.0).0) as u8) | (((center != (area.2).0) as u8) << 1) | (((center != (area.0).2) as u8) << 2) | (((center != (area.2).2) as u8) << 3);
		
		// If it can't spill, set the entire mask to 0. This results in mask.trailing_zeros() returning 8 assuming it's type is u8 (not 0!).
		mask &= (can_spill as u8) * 15;
		
		match mask.trailing_zeros() {
			0 => (area.0).0,
			1 => (area.2).0,
			2 => (area.0).2,
			3 => (area.2).2,
			_ => center
		}
	}
}

pub trait Mix<T> {
	fn spill_chance(&self, value: &T) -> i32;
}

pub struct BoolMix {
	pub true_chance: i32,
	pub false_chance: i32
}

impl Mix<bool> for BoolMix {
	fn spill_chance(&self, value: &bool) -> i32 {
		if *value {self.true_chance} else {self.false_chance}
	}
}