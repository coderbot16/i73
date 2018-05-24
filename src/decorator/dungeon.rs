use java_rand::Random;

// TODO
#[derive(Debug, Copy, Clone)]
pub enum Item {
	Saddle,
	IronIngot,
	Bread,
	Wheat,
	Gunpowder,
	String,
	Bucket,
	GoldenApple,
	Redstone,
	GoldRecord,
	GreenRecord,
	InkSac
}

#[derive(Debug)]
pub struct Stack {
	item: Item,
	size: u32
}

pub struct SimpleLootTable {
	pools: Vec<Pool>
}

impl SimpleLootTable {
	pub fn get_item(&self, rng: &mut Random) -> Option<Stack> {
		if self.pools.len() != 0 {
			self.pools[rng.next_u32_bound(self.pools.len() as u32) as usize].get_item(rng)
		} else {
			None
		}
	}
}

impl Default for SimpleLootTable {
	fn default() -> Self {
		SimpleLootTable {
			pools: vec![
				Pool::Common { item: Item::Saddle,    base_size: 1, add_size: None    },
				Pool::Common { item: Item::IronIngot, base_size: 1, add_size: Some(3) },
				Pool::Common { item: Item::Bread,     base_size: 1, add_size: None    },
				Pool::Common { item: Item::Wheat,     base_size: 1, add_size: Some(3) },
				Pool::Common { item: Item::Gunpowder, base_size: 1, add_size: Some(3) },
				Pool::Common { item: Item::String,    base_size: 1, add_size: Some(3) },
				Pool::Common { item: Item::Bucket,    base_size: 1, add_size: None    },
				Pool::rare   ( Pool::single(Item::GoldenApple), 100),
				Pool::rare   ( Pool::Common { item: Item::Redstone, base_size: 1, add_size: Some(3) }, 2),
				Pool::rare   ( Pool::Table(records()), 10),
				Pool::Common { item: Item::InkSac,    base_size: 1, add_size: None    }
			]
		}
	}
}

fn records() -> SimpleLootTable {
	SimpleLootTable {
		pools: vec![
			Pool::single(Item::GoldRecord),
			Pool::single(Item::GreenRecord)
		]
	}
}

enum Pool {
	/// Creates an item with a stack size of base + rng(add + 1)
	Common { item: Item, base_size: u32, add_size: Option<u32> },
	Decide { item: Box<Pool>, other: Option<Box<Pool>>, chance: u32 },
	Table  (SimpleLootTable)
}

impl Pool {
	fn single(item: Item) -> Self {
		Pool::Common { item, base_size: 1, add_size: None }
	}
	
	fn rare(item: Pool, chance: u32) -> Self {
		Pool::Decide { item: Box::new(item), other: None, chance }
	}
	
	fn get_item(&self, rng: &mut Random) -> Option<Stack> {
		match *self {
			Pool::Common { ref item, base_size, add_size } => Some(Stack { 
				item: *item, 
				size: base_size + add_size.map(|i| rng.next_u32_bound(i + 1)).unwrap_or(0)
			}),
			Pool::Decide { ref item, ref other, chance   } => if rng.next_u32_bound(chance) == 0 {
				item.get_item(rng)
			} else {
				other.as_ref().and_then(|o| o.get_item(rng))
			},
			Pool::Table  ( ref table ) => table.get_item(rng)
		}
	}
}

pub enum SpawnerMob {
	Skeleton,
	Zombie,
	Spider
}

impl SpawnerMob {
	pub fn select(rng: &mut Random) -> Self {
		match rng.next_u32_bound(4) {
			0     => SpawnerMob::Skeleton,
			1 | 2 => SpawnerMob::Zombie,
			3     => SpawnerMob::Spider,
			_     => unreachable!()
		}
	}
}