use rng::NotchRng;
use std::cmp::max;
use image_ops::filter::Filter;
use image_ops::Image;

/// The Zoom filter resamples the input image to be 2 times larger on each axis.
pub struct Zoom<T, S> where S: SelectZoom<T> {
	rng: NotchRng,
	selector: S,
	marker: ::std::marker::PhantomData<T>
}

impl<T, S> Zoom<T, S> where S: SelectZoom<T> {
	pub fn new(rng: NotchRng, selector: S) -> Self {
		Zoom {
			rng,
			selector,
			marker: ::std::marker::PhantomData
		}
	}
}

impl<T, S> Filter<T> for Zoom<T, S> where S: SelectZoom<T>, T: Default + Copy {
	type Out = T;
	
	fn filter(&self, position: (i64, i64), image: &Image<T>, out: &mut Image<Self::Out>) {
		let aligned_position = (position.0 & (!1), position.1 & (!1));
		
		let aligned_size = (out.x_size() & !1, out.z_size() & !1);
		
		let sample_size = self.input_size((out.x_size(), out.z_size()));
		
		if image.x_size() < sample_size.0 {
			panic!("Source image X size of {} is less than the required minimum X size {} for zooming in to a target X size of {}", image.x_size(), sample_size.0, out.x_size());
		}
		
		if image.z_size() < sample_size.1 {
			panic!("Source image Z size of {} is less than the required minimum Z size {} for zooming in to a target Z size of {}", image.z_size(), sample_size.1, out.z_size());
		}
		
		// TODO: Write to out directly, unaligning with array access.
		let mut aligned = Image::new(T::default(), aligned_size.0 + 6, aligned_size.1 + 6);
		
		for z in 0..sample_size.0-1 {
			for x in 0..sample_size.0-1 {
				let mut rng = self.rng.clone();
				rng.init_at(aligned_position.0 + (x as i64) * 2, aligned_position.1 + (z as i64) * 2);
				
				let values = (
					*image.get(x,     z    ),
					*image.get(x + 1, z    ),
					*image.get(x,     z + 1),
					*image.get(x + 1, z + 1),
				);
				
				let aligned_z = z * 2;
				let aligned_x = x * 2;
				
				aligned.set(aligned_x,     aligned_z,     values.0);
				aligned.set(aligned_x,     aligned_z + 1, if rng.next_i32(2)==0 {values.0} else {values.2});
				aligned.set(aligned_x + 1, aligned_z,     if rng.next_i32(2)==0 {values.0} else {values.1});
				aligned.set(aligned_x + 1, aligned_z + 1, self.selector.select(&mut rng, values));
			}
		}
		
		// Unalign the image. The aligned image is aligned to an even value on the X and Z axis, but the caller may have requested from an odd position.
		let x_add = (position.0 & 1) as usize;
		let z_add = (position.1 & 1) as usize;
		
		for z in 0..out.z_size() {
			for x in 0..out.x_size() {
				out.set(x, z, *aligned.get(x + x_add, z + z_add));
			}
		}
	}
	
	fn input_size(&self, size: (usize, usize)) -> (usize, usize) {
		(
			(size.0 / 2) + 3,
			(size.1 / 2) + 3
		)
	}
	
	fn input_position(&self, position: (i64, i64)) -> (i64, i64) {
		(
			position.0 >> 1,
			position.1 >> 1
		)
	}
}

pub trait SelectZoom<T> {
	fn select(&self, rng: &mut NotchRng, options: (T, T, T, T)) -> T;
}

/// Randomly chooses from the available options. This is the least accurate option, but works with every type.
pub struct RandomCandidate;

impl<T> SelectZoom<T> for RandomCandidate {
	fn select(&self, rng: &mut NotchRng, options: (T, T, T, T)) -> T {
		match rng.next_i32(4) {
			0 => options.0,
			1 => options.1,
			2 => options.2,
			3 => options.3,
			_ => unreachable!()
		}
	}
}

/// Selects the best possible candidate of a set of options. If there are other options with the same number of elements that they equal, then falls back to randomly choosing.
/// A way to select candidates without using blending, needed when not working with non blendable values such as biome IDs.
pub struct BestCandidate;

impl<T> SelectZoom<T> for BestCandidate where T: Eq {
	fn select(&self, rng: &mut NotchRng, options: (T, T, T, T)) -> T {
		let picked = rng.next_i32(4) as u32;
		
		// Instead of using a ton of if statements that resembles puke more than code,
		// Use branchless algorithms that are also way faster.
		
		let sums = (
			((options.0 == options.1) as u8) + ((options.0 == options.2) as u8) + ((options.0 == options.3) as u8),
			((options.1 == options.0) as u8) + ((options.1 == options.2) as u8) + ((options.1 == options.3) as u8),
			((options.2 == options.0) as u8) + ((options.2 == options.1) as u8) + ((options.2 == options.3) as u8),
			((options.3 == options.0) as u8) + ((options.3 == options.1) as u8) + ((options.3 == options.2) as u8)
		);
		
		let max_sum = max(
			max(sums.0, sums.1),
			max(sums.2, sums.3)
		);
		
		let selectable = (sums.0 == max_sum, sums.1 == max_sum, sums.2 == max_sum, sums.3 == max_sum);
		
		let mask = (selectable.0 as u8) | ((selectable.1 as u8) << 1) | ((selectable.2 as u8) << 2) | ((selectable.3 as u8) << 3);
		
		let first = mask.trailing_zeros();
		let num_selectable = mask.count_ones();
		
		// If all values are equal, we still use the randomly picked value anyway for brevity. If all of the values are equal, the index doesn't matter.
		let index = if num_selectable == 4 {picked} else {first};
		
		// We have to use a tuple, as arrays do not allow moves out.
		match index {
			0 => options.0,
			1 => options.1,
			2 => options.2,
			3 => options.3,
			_ => unreachable!()
		}
	}
}

/// Will it blend?
/// 
/// Yes, it will.
struct FuzzyBlend {
	rng: NotchRng
}