//! Types for matching against specific block.
//! TODO: Replace with sparse bit array in `vocs`.
//! Generic types are not configurable and are a band aid.
//! A component-based solution, in comparison, would be much more configurable.

use vocs::indexed::Target;
use std::collections::HashSet;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag="kind")]
pub enum BaselineMatcher<B> where B: Target {
	Whitelist { blocks: HashSet<B> },
	Blacklist { blocks: HashSet<B> }
}

impl<B> BlockMatcher<B> for BaselineMatcher<B> where B: Target {
	fn matches(&self, block: &B) -> bool {
		match self {
			&BaselineMatcher::Whitelist { ref blocks } => blocks.contains(block),
			&BaselineMatcher::Blacklist { ref blocks } => !blocks.contains(block)
		}
	}
}

// --

pub trait BlockMatcher<B> where B: Target {
	fn matches(&self, block: &B) -> bool;
}

impl<T, B> BlockMatcher<B> for T where T: Fn(&B) -> bool, B: Target {
	fn matches(&self, block: &B) -> bool {
		self(block)
	}
}

#[derive(Debug, Clone)]
pub struct HashSetMatcher<B>(pub ::std::collections::HashSet<B>) where B: Target;
impl<B> BlockMatcher<B> for HashSetMatcher<B> where B: Target {
	fn matches(&self, block: &B) -> bool {
		self.0.contains(block)
	}
}

#[derive(Debug, Copy, Clone)]
pub struct All;

impl<B> BlockMatcher<B> for All where B: Target {
	fn matches(&self, _block: &B) -> bool {
		true
	}
}

#[derive(Debug, Copy, Clone)]
pub struct None;

impl<B> BlockMatcher<B> for None where B: Target {
	fn matches(&self, _block: &B) -> bool {
		false
	}
}

#[derive(Debug, Copy, Clone)]
pub struct Is<B>(pub B) where B: Target;

impl<B> BlockMatcher<B> for Is<B> where B: Target {
	fn matches(&self, block: &B) -> bool {
		*block == self.0
	}
}

#[derive(Debug, Copy, Clone)]
pub struct IsNot<B>(pub B) where B: Target;

impl<B> BlockMatcher<B> for IsNot<B> where B: Target {
	fn matches(&self, block: &B) -> bool {
		*block != self.0
	}
}