use chunk::storage::Target;
use chunk::grouping::{Column, Result};

pub mod overworld_173;
pub mod nether_173;
pub mod sky_173;

pub trait Pass<B> where B: Target {
	fn apply(&self, target: &mut Column<B>, chunk: (i32, i32)) -> Result<()>;
}