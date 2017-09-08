#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum Block {
	// L1 blocks: Generated in the ShapeTerrain pass.
	Air,
	Stone,
	Water,
	Ice,
	// L2 blocks: Generated in the PaintSurface pass.
	Bedrock,
	Dirt,
	Grass,
	Sand,
	Sandstone,
	Gravel,
	// L3 blocks: Generated in the Decorate pass.
}