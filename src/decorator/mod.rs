use rng::JavaRng;

pub mod dungeon;
pub mod vein;

// TODO
type Moore = ();

trait Decorator {
	fn generate(&self, moore: &mut Moore, rng: &mut JavaRng, position: (i32, i32, i32));
}