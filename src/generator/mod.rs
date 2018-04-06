use vocs::indexed::Target;
use vocs::world::view::ColumnMut;
use vocs::position::GlobalColumnPosition;

pub mod overworld_173;
pub mod nether_173;
pub mod sky_173;

pub trait Pass<B> where B: Target {
	fn apply(&self, target: &mut ColumnMut<B>, chunk: GlobalColumnPosition);
}