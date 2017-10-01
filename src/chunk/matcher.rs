use chunk::storage::Target;

pub trait BlockMatcher<B> where B: Target {
	fn matches(&self, block: &B) -> bool;
}

impl<T, B> BlockMatcher<B> for T where T: Fn(&B) -> bool, B: Target {
	fn matches(&self, block: &B) -> bool {
		self(block)
	}
}

#[derive(Copy, Clone)]
pub struct All;

impl<B> BlockMatcher<B> for All where B: Target {
	fn matches(&self, _block: &B) -> bool {
		true
	}
}

#[derive(Copy, Clone)]
pub struct None;

impl<B> BlockMatcher<B> for None where B: Target {
	fn matches(&self, _block: &B) -> bool {
		false
	}
}