#[macro_use]
extern crate nom;

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate nbt_serde;
extern crate byteorder;
extern crate deflate;
extern crate bit_vec;
extern crate vocs;
extern crate nalgebra;

pub mod noise;
pub mod rng;
pub mod biome;
pub mod sample;
pub mod noise_field;
pub mod decorator;
pub mod trig;
pub mod structure;
pub mod generator;
pub mod distribution;
pub mod segmented;
pub mod config;
pub mod matcher;

#[cfg(test)]
mod test {
	use std::fs::File;
	use std::io::Read;

	pub fn read_u64s(name: &str) -> Vec<u64> {
		let mut file = File::open(format!("test_data/{}.txt", name)).unwrap();
		let mut data = Vec::new();

		let mut term = 0u64;
		for byte in file.bytes() {
			let c = (byte.unwrap() as char).to_lowercase().next().unwrap();

			if c.is_whitespace() {
				data.push(term);

				term = 0;
			} else {
				term <<= 4;
				term |= if c >= '0' && c <= '9' {
					(c as u64) - ('0' as u64)
				} else if c >= 'a' && c <= 'f' {
					(c as u64) - ('a' as u64) + 10
				} else {
					panic!("Bad hex character {}", c);
				};
			}
		}

		data.push(term);

		data
	}

	pub fn read_u32s(name: &str) -> Vec<u32> {
		read_u64s(name).iter().map(|&v| v as u32).collect::<Vec<_>>()
	}

	pub fn read_f32s(name: &str) -> Vec<f32> {
		read_u64s(name).iter().map(|&v| f32::from_bits(v as u32)).collect::<Vec<_>>()
	}

	pub fn read_f64s(name: &str) -> Vec<f64> {
		read_u64s(name).iter().map(|&v| f64::from_bits(v)).collect::<Vec<_>>()
	}
}