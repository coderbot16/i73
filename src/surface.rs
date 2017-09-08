use block::Block;
use biome::Biome;

const SEA_COORD:  u32 = 63;
const BEACH_LOW:  u32 = SEA_COORD - 3;
const BEACH_HIGH: u32 = SEA_COORD + 2;

pub struct Surface {
	pub top:  Option<Block>,
	pub fill: Block
}

enum Beach {
	Sand,
	Gravel,
	Biome
}

impl Beach {
	fn surface(&self, biome: &Biome) -> Surface {
		match *self {
			Beach::Sand   => Surface { top: Some(Block::Sand), fill: Block::Sand },
			Beach::Gravel => Surface { top: None, fill: Block::Gravel },
			Beach::Biome  => biome.surface()
		}
	}
}

struct Stack {
	depth: i32,
	beach: Beach,
	biome: Biome
}

impl Stack {
	fn surface(&self, y: u32, last: &Surface) -> Surface {
		let mut surface = if self.depth <= 0 {
			Surface { top: None, fill: Block::Stone }
		} else if y >= BEACH_LOW && y <= BEACH_HIGH {
			self.beach.surface(&self.biome)
		} else {
			Surface { top: self.biome.surface().top, fill: last.fill }
		};
		
		if y < SEA_COORD {
			surface.top = Some(surface.fill);
		}
		
		surface
	}
}