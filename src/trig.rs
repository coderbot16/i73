include!(concat!(env!("OUT_DIR"), "/sin_table.rs"));

pub fn sin(f: f32) -> f32 {
	sin_index(((f * 10430.38) as i32) as u32)
}

pub fn cos(f: f32) -> f32 {
	sin_index(((f * 10430.38 + 16384.0) as i32) as u32)
}

fn sin_index(idx: u32) -> f32 {
	let idx = idx & 0xFFFF;

	let neg = (idx & 0x8000) << 16;
	let idx2 = idx & 0x7FFF;
	let invert = (idx & 0x4000) >> 14;

	let full_invert = 0u32.wrapping_sub(invert);
	let sub_from = (invert << 15) + invert;
	let idx3 = ::std::cmp::min(sub_from.wrapping_add(idx2 ^ full_invert), 16383);

	let wierd = (idx == 32768) as u32;

	let raw = (SIN_TABLE[idx3 as usize] ^ neg).wrapping_add(wierd * 0xA50D3132);

	f32::from_bits(raw)
}

#[cfg(test)]
mod test {
	#[test]
	fn test_sin() {
		let java = ::test::read_u32s("JavaSinTable");

		assert_eq!(java.len(), 65536);

		for index in 0..65536 {
			let r = super::sin_index(index).to_bits();
			let j = java[index as usize];

			if r != j {
				panic!("trig::test_sin: mismatch @ index {}: {} (R) != {} (J)", index, r, j);
			}
		}
	}
}