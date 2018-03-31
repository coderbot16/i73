use vocs::world::chunk::Target;
use vocs::world::view::ColumnMut;

pub mod overworld_173;
pub mod nether_173;
pub mod sky_173;

pub trait Pass<B> where B: Target {
	fn apply(&self, target: &mut ColumnMut<B>, chunk: (i32, i32));
}