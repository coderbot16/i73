use decorator::Decorator;
use rng::JavaRng;

struct Scattering<D: Decorator> {
	iterations: usize,
	horizontal: i32,
	vertical: i32,
	decorator: D
}

impl<D> Decorator for Scattering<D> where D: Decorator {
	fn generate(&self, moore: &mut Moore, rng: &mut JavaRng, position: (i32, i32, i32)) {
		let (mut x, mut y, mut z) = position;
		
		for _ in 0..self.iterations {
			x += rng.next_i32(self.horizontal) - rng.next_i32(self.horizontal);
			y += rng.next_i32(self.vertical)   - rng.next_i32(self.vertical);
			z += rng.next_i32(self.horizontal) - rng.next_i32(self.horizontal);
			
			self.decorator.generate(moore, rng, (x, y, z));
		}
	}
}

struct HorizontalScattering<D: Decorator> {
	iterations: usize,
	horizontal: i32,
	decorator: D
}

impl<D> Decorator for HorizontalScattering<D> where D: Decorator {
	fn generate(&self, moore: &mut Moore, rng: &mut JavaRng, position: (i32, i32, i32)) {
		let (mut x, mut y, mut z) = position;
		
		for _ in 0..self.iterations {
			x += rng.next_i32(self.horizontal) - rng.next_i32(self.horizontal);
			z += rng.next_i32(self.horizontal) - rng.next_i32(self.horizontal);
			
			self.decorator.generate(moore, rng, (x, y, z));
		}
	}
}

// TYPES

struct Cactus {
	/// Base, minimum height of a cactus
	base_height: i32,
	/// Maximum height of a cactus when added to the base height. 
	/// For example, with base=1 and add=2, the height of a cactus can be 1-3 blocks tall.
	add_height: i32
}

impl Cactus {
	fn check(&self, moore: &mut Moore, position: (i32, i32, i32)) -> boolean {
		if moore.get(position) != Block::Air {
			false
		} else if moore.get((position.0 - 1, position.1, position.2)).is_solid() {
			false
		} else if moore.get((position.0 + 1, position.1, position.2)).is_solid() {
			false
		} else if moore.get((position.0, position.1, position.2 - 1)).is_solid() {
			false
		} else if moore.get((position.0, position.1, position.2 + 1)).is_solid() {
			false
		} else {
			let block =  moore.get((position.0, position.1 - 1, position.2));
			block == Block::Sand || block == Block::Cactus
		}
	}
}

impl Decorator for Cactus {
	fn generate(&self, moore: &mut Moore, rng: &mut JavaRng, position: (i32, i32, i32)) {
		if moore.get(position) != Block::Air {
			return;
		}
		
		let height = self.base_height + rng.next_i32(rng.next_i32(self.add_height + 1) + 1);
		
		for y in 0..height {
			let pos = (position.0, position.1 + y, position.2);
			
			if self.check(moore, pos) {
				moore.set(pos, Block::Cactus);
			}
		}
	}
}

fn cactus() -> Scattering<Cactus> {
	Scattering {
		iterations: 10,
		horizontal: 8,
		vertical: 4,
		decorator: Cactus {
			base_height: 1,
			add_height: 2
		}
	}
}