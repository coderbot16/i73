use image_ops::Image;
use std::borrow::Cow;

pub trait Source {
	type Out;
	
	fn fill(&self, position: (i64, i64), image: &mut Image<Self::Out>);
}

pub trait Filter<T> {
	type Out;
	
	fn filter(&self, position: (i64, i64), image: &Image<T>, out: &mut Image<Self::Out>);
	fn input_size(&self, size: (usize, usize)) -> (usize, usize);
	fn input_position(&self, position: (i64, i64)) -> (i64, i64);
}

pub trait Mix<T, U> {
	type Out;
	
	fn filter(&self, position: (i64, i64), first: &Image<T>, second: &Image<U>) -> Image<Self::Out>;
}

pub trait Map<T> {
	type Out;
	
	fn map(&self, position: (i64, i64), value: T) -> Self::Out;
}

/// Chains a set of filters together.
pub struct Chain<T>(pub Vec<Box<Filter<T, Out=T>>>);

impl<T> Chain<T> {
	pub fn new() -> Self {
		Chain(Vec::new())
	}
}

impl<T> Filter<T> for Chain<T> where T: Default + Clone {
	type Out = T;
	
	fn filter(&self, position: (i64, i64), image: &Image<T>, out: &mut Image<Self::Out>) {
		if self.0.is_empty () {
			for z in 0..out.z_size() {
				for x in 0..out.x_size() {
					out.set(x, z, image.get(x, z).clone());
				}
			}
			
			return;
		}
		
		let mut image = Cow::Borrowed(image);
		
		for (index, filter) in self.0.iter().enumerate() {
			let mut cur_size = out.size();
			let mut out_size = out.size();
			let mut cur_position = position;
			
			for filter in self.0.iter().rev().take(self.0.len() - index) {
				cur_size = filter.input_size(cur_size);
			}
			
			if index < self.0.len() - 1 {
				for filter in self.0.iter().rev().take(self.0.len() - 1 - index) {
					out_size = filter.input_size(out_size);
					cur_position = filter.input_position(cur_position);
				}
			}
			
			println!("[{}]: InSz=({:?} expected, {:?} real) OutSz={:?} InPos={:?}", index, cur_size, image.size(), out_size, cur_position);
			
			if index < self.0.len() - 1 {
				let mut out = Image::new(T::default(), out_size.0, out_size.1);
			
				filter.filter(cur_position, &image, &mut out);
			
				image = Cow::Owned(out);
			} else {
				println!("filter direct @ {:?}", cur_position);
				
				filter.filter(cur_position, &image, out);
			}
		}
	}
	
	fn input_size(&self, size: (usize, usize)) -> (usize, usize) {
		let mut size = size;
		
		for filter in self.0.iter().rev() {
			size = filter.input_size(size);
		}
		
		size
	}
	
	fn input_position(&self, position: (i64, i64)) -> (i64, i64) {
		let mut position = position;
		
		if !self.0.is_empty() {
			for filter in self.0.iter().rev().take(self.0.len()) {
				position = filter.input_position(position);
			}
		}
		
		position
	}
}