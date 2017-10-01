#[derive(Clone, Debug)]
pub struct Segmented<T> {
	segments: Vec<Segment<T>>,
	out: T
}

impl<T> Segmented<T> {
	pub fn new(default: T) -> Self {
		Segmented {
			segments: Vec::new(),
			out: default
		}
	}
	
	pub fn default_out(&self) -> &T {
		&self.out
	}
	
	fn segment(&self, at: f64, always_inclusive: bool) -> Option<(&Segment<T>, usize)> {
		let last_idx = if self.segments.len() > 0 { Some(self.segments.len() - 1) } else { None };
		
		for (index, segment) in self.segments.iter().enumerate() {
			if (at < segment.upper) || ((Some(index) == last_idx || always_inclusive) && at == segment.upper) {
				return Some((segment, index));
			}
		}
		
		None
	}
	
	fn segment_mut(&mut self, at: f64, always_inclusive: bool) -> Option<(&mut Segment<T>, usize)> {
		let last_idx = if self.segments.len() > 0 { Some(self.segments.len() - 1) } else { None };
		
		for (index, segment) in self.segments.iter_mut().enumerate() {
			if (at < segment.upper) || ((Some(index) == last_idx || always_inclusive) && at == segment.upper) {
				return Some((segment, index));
			}
		}
		
		None
	}
	
	pub fn add_boundary(&mut self, upper: f64, value: T) {
		if let Some((seg_upper, index)) = self.segment(upper, true).map(|(seg, index)| (seg.upper, index)) {
			if seg_upper == upper {
				self.segment_mut(upper, true).unwrap().0.value = value;
			} else {
				self.segments.insert(index, Segment { upper, value });
			}
		} else {
			let len = self.segments.len();
			self.segments.insert(len, Segment { upper, value });
		}
	}
	
	pub fn get(&self, at: f64) -> &T {
		self.segment(at, false).map(|seg| &seg.0.value).unwrap_or(&self.out)
	}
}

impl<T> Segmented<T> where T: Clone {
	pub fn for_all_aligned<A, F>(&mut self, lower: f64, upper: f64, above: &A, on: &F) where A: Fn() -> T, F: Fn(&mut T) {
		self.align(lower, upper, above);
		
		let mut last_upper = None;
		
		for segment in &mut self.segments {
			let above = last_upper.map(|last_upper| lower <= last_upper).unwrap_or(true);
			
			if above && lower < segment.upper && segment.upper <= upper {
				on(&mut segment.value)
			}
			
			last_upper = Some(segment.upper);
		}
	}
	
	/// Makes sure that 1 segment has an upper equal to the lower and 1 segment has an upper equal to the upper.
	pub fn align<F>(&mut self, lower: f64, upper: f64, above: &F) where F: Fn() -> T {
		let split_index = self.segment(lower, true).map(|(_, index)| index).unwrap_or(self.segments.len() - 1);
		self.split(split_index, lower, above);
		
		let end_index = self.segment(upper, true).map(|(_, index)| index).unwrap_or(self.segments.len() - 1);
		self.split(end_index, upper, above);
	}
	
	fn split<F>(&mut self, index: usize, new_boundary: f64, above: &F) where F: Fn() -> T {
		if self.segments[index].upper == new_boundary {
			return;
		}
		
		let before = self.segments[index].upper > new_boundary;
		
		let (index, value) = if before {
			(index, self.segments[index].value.clone())
		} else {
			(index + 1, above())
		};
		
		self.segments.insert(index, Segment { upper: new_boundary, value })
	}
}

impl<T> Segmented<Segmented<T>> where T: Clone {
	// TODO: insert a rect
}

#[derive(Clone, Debug)]
struct Segment<T> {
	upper: f64,
	value: T
}