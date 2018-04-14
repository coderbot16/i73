
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;

fn main() {
	let out_dir = env::var("OUT_DIR").unwrap();
	let dest_path = Path::new(&out_dir).join("sin_table.rs");
	let mut f = File::create(&dest_path).unwrap();

	println!("cargo:rerun-if-changed=build.rs");

	write!(f, "pub const SIN_TABLE: [u32; 16384] = [").unwrap();

	for i in 0..16384 {
		if i % 16 == 0 {
			writeln!(f, "\t").unwrap();
		}

		write!(f, "{:#010X}, ", trig(i).to_bits()).unwrap();
	}

	writeln!(f, "];").unwrap();
}

fn trig(index: u16) -> f32 {
	((index as f64) * 3.141592653589793 * 2.0 / 65536.0).sin() as f32
}