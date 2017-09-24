use chunk::storage::Target;

pub trait BlockMatcher<B> where B: Target {
	fn matches(&self, block: &B) -> bool;
}

impl<T, B> BlockMatcher<B> for T where T: Fn(&B) -> bool, B: Target {
	fn matches(&self, block: &B) -> bool {
		self(block)
	}
}