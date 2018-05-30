//! Types for matching against specific block.
//! TODO: Replace with sparse bit array in `vocs`.
//! Generic types are not configurable and are a band aid.
//! A component-based solution, in comparison, would be much more configurable.

use vocs::indexed::Target;
use std::collections::HashSet;
use std::iter::{IntoIterator, FromIterator, Iterator};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BlockMatcher<B> where B: Target {
	pub blocks: HashSet<B>,
	pub blacklist: bool
}

impl<B> BlockMatcher<B> where B: Target {
	pub fn is(block: B) -> Self {
		let mut blocks = HashSet::with_capacity(1);
		blocks.insert(block);

		BlockMatcher {
			blocks,
			blacklist: false
		}
	}

	pub fn is_not(block: B) -> Self {
		let mut blocks = HashSet::with_capacity(1);
		blocks.insert(block);

		BlockMatcher {
			blocks,
			blacklist: true
		}
	}

	pub fn include<'a, I>(blocks: I) -> Self where I: IntoIterator<Item=&'a B>, B: 'a {
		BlockMatcher {
			blocks: HashSet::from_iter(blocks.into_iter().map(|x| x.clone())),
			blacklist: false
		}
	}

	pub fn exclude<'a, I>(blocks: I) -> Self where I: IntoIterator<Item=&'a B>, B: 'a{
		BlockMatcher {
			blocks: HashSet::from_iter(blocks.into_iter().map(|x| x.clone())),
			blacklist: true
		}
	}

	pub fn matches(&self, block: &B) -> bool {
		// NotPresent, Whitelist => 0 ^ 0 => 0
		// NotPresent, Blacklist => 0 ^ 1 => 1
		// Contains, Whitelist => 1 ^ 0 => 1
		// Contains, Blacklist => 1 ^ 1 => 0
		self.blocks.contains(block) ^ self.blacklist
	}
}

impl<B> DeprecatedBlockMatcher<B> for BlockMatcher<B> where B: Target {
	fn matches(&self, block: &B) -> bool {
		BlockMatcher::matches(self, block)
	}
}

// --

pub trait DeprecatedBlockMatcher<B> where B: Target {
	fn matches(&self, block: &B) -> bool;
}

impl<T, B> DeprecatedBlockMatcher<B> for T where T: Fn(&B) -> bool, B: Target {
	fn matches(&self, block: &B) -> bool {
		self(block)
	}
}