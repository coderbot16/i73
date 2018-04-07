use std::str::{self, FromStr};
use nom::{digit, IError};
use std::collections::HashMap;

// V1: 1; 
//     [<count>x]<numeric_id>[:<damage>], ...; 
//     <biome>

// V2: 2; 
//     [<count>x]<numeric_id>[:<damage>], ...; 
//     <biome>;
//     <structure_name>[(<param>=<value>, ...)], ...

// V3: 3; 
//     [<count>*][<namespace>:]<id>[:<damage>], ...; 
//     <biome>;
//     <structure_name>[(<param>=<value>, ...)], ...

#[derive(Debug, PartialEq)]
pub struct FlatV1 {
	pub layers: Vec<LayerV1>,
	pub biome: i64
}

impl FromStr for FlatV1 {
	type Err = IError;
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		parse_flat_v1(s).to_full_result()
	}
}

#[derive(Debug, PartialEq)]
pub struct LayerV1 {
	count: i64,
	id: i64,
	meta: i64
}

named!(parse_flat_v1<&str, FlatV1>,
	do_parse!(
		version: tag!("1;") >>
		layers: many1!(parse_layer_v1) >>
		biome: alt!(parse_biome_v1 | value!(1)) >>
		(FlatV1 { layers, biome })
	)
);

named!(parse_biome_v1<&str, i64>,
	do_parse!(
		marker: tag!(";") >>
		biome: integer >>
		(biome)
	)
);

named!(parse_layer_v1<&str, LayerV1>, 
	do_parse!(
		count: alt!(count_v1 | value!(1)) >>
		id: integer >>
		meta: alt!(meta_v1 | value!(0)) >>
		sep: alt!(tag!(",") | tag!("")) >>
		(LayerV1 { count, id, meta })
	)
);

named!(count_v1<&str, i64>,
	do_parse!(
		count: integer >>
		marker: tag!("x") >>
		(count)
	)
);

named!(meta_v1<&str, i64>,
	do_parse!(
		marker: tag!(":") >>
		meta: integer >>
		(meta)
	)
);

named!(integer<&str, i64>,
	map_res!(
		 digit,
		 FromStr::from_str
	)
);

#[derive(Debug, PartialEq)]
pub struct FlatV3 {
	pub layers: Vec<LayerV3>,
	pub biome: i64,
	pub features: HashMap<String, HashMap<String, String>>
}

impl FlatV3 {
	pub fn add_feature(&mut self, name: &str) {
		self.features.insert(name.to_owned(), HashMap::new());
	}
}

impl FromStr for FlatV3 {
	type Err = IError;
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		parse_flat_v3(s).to_full_result()
	}
}

#[derive(Debug, PartialEq)]
pub struct LayerV3 {
	count: i64,
	namespace: String,
	id: String,
	meta: i64
}

impl LayerV3 {
	fn from_parts(count: i64, parts: (&str, &str, i64)) -> Self {
		LayerV3 {
			count,
			namespace: parts.0.to_string(),
			id: parts.1.to_string(),
			meta: parts.2
		}
	}
}

fn features_to_map(input: Vec<(String, Vec<(String, String)>)>) -> HashMap<String, HashMap<String, String>> {
	let mut out = HashMap::with_capacity(input.len());
	
	for feature in input {
		let mut options = HashMap::with_capacity(feature.1.len());
		
		for option in feature.1 {
			options.insert(option.0, option.1);
		}
		
		out.insert(feature.0, options);
	}
	
	out
}

named!(parse_flat_v3<&str, FlatV3>,
	do_parse!(
		version: tag!("3;") >>
		layers: many1!(parse_layer_v3) >>
		biome: alt!(parse_biome_v1 | value!(1)) >>
		features: alt!( 
			preceded!( tag!(";"), many_till!( call!(parse_feature_v3), call!(eof_thunk) )) | 
			value!((Vec::new(), ""))
		) >>
		(FlatV3 { layers, biome, features: features_to_map(features.0) })
	)
);

named!(eof_thunk<&str, &str>, eof!());

named!(parse_layer_v3<&str, LayerV3>, 
	do_parse!(
		count: alt!(count_v3 | value!(1)) >>
		block: alt!(parse_id_all | parse_name_meta | parse_namespaced | parse_id_name) >>
		(LayerV3::from_parts(count, block))
	)
);

named!(parse_id_all<&str, (&str, &str, i64)>,
	do_parse!(
		namespace: take_till!(|c| c==':' || c==',' || c==';') >>
		sep0: tag!(":") >>
		id: take_till!(|c| c==':' || c==',' || c==';') >>
		sep1: tag!(":") >>
		meta: integer >>
		sep2: alt!(tag!(",") | tag!("")) >>
		((namespace, id, meta))
	)
);

named!(parse_name_meta<&str, (&str, &str, i64)>,
	do_parse!(
		id: take_till!(|c| c==':' || c==',' || c==';') >>
		sep0: tag!(":") >>
		meta: integer >>
		sep1: alt!(tag!(",") | tag!("")) >>
		(("minecraft", id, meta))
	)
);

named!(parse_namespaced<&str, (&str, &str, i64)>,
	do_parse!(
		namespace: take_till!(|c| c==':' || c==',' || c==';') >>
		sep0: tag!(":") >>
		id: take_till!(|c| c==':' || c==',' || c==';') >>
		sep1: alt!(tag!(",") | tag!("")) >>
		((namespace, id, 0))
	)
);

named!(parse_id_name<&str, (&str, &str, i64)>,
	do_parse!(
		id: take_till!(|c| c==',' || c==';') >>
		sep1: alt!(tag!(",") | tag!("")) >>
		(("minecraft", id, 0))
	)
);

// Parses a count of the form <COUNT>*
named!(count_v3<&str, i64>,
	do_parse!(
		count: integer >>
		marker: tag!("*") >>
		(count)
	)
);

// Parses a feature of the format <NAME>(<KEY=VALUE>, <KEY=VALUE>, ..) or just plain <NAME>.
named!(parse_feature_v3<&str, (String, Vec<(String, String)>)>,
	do_parse!(
		name: take_till!(|c| c==',' || c=='(' || c==';') >>
		options: parse_feature_options_v3 >>
		sep: alt!(eof!() | tag!(",") | tag!("")) >>
		(name.to_string(), options)
	)
);

// Parses an option list of the format (<KEY=VALUE>, <KEY=VALUE>, ..)
named!(parse_feature_options_v3<&str, Vec<(String, String)>>, alt!(
	preceded!(
		alt!(eof!() | tag!(",")), 
		value!(Vec::new())
	) | do_parse!(
		open: tag!("(") >>
		options: many_till!(call!(parse_option_v3), tag!(")") ) >>
		(options.0)
	)
));

// Parses an option of the format <KEY>=<VALUE>, and ends at either a space, seperating kv pairs, or a ')', which ends the map.
named!(parse_option_v3<&str, (String, String)>,
	do_parse!(
		key: take_until_and_consume!("=") >>
		value: take_till!(|c| c==' ' || c==')') >>
		sep: alt!(tag!(" ") | tag!("")) >>
		((key.to_string(), value.to_string()))
	)
);

#[cfg(test)]
mod test {
	use super::{FlatV1, LayerV1, FlatV3, LayerV3};
	use std::collections::HashMap;
	
	#[test]
	fn test_default_v1() {
		let default = FlatV1 {
			layers: vec![
				LayerV1 { count: 1, id: 7, meta: 0 },
				LayerV1 { count: 2, id: 3, meta: 0 },
				LayerV1 { count: 1, id: 2, meta: 0 }
			],
			biome: 1
		};
		
		assert_eq!(Ok(default), "1;7,2x3,2;1".parse::<FlatV1>())
	}
	
	#[test]
	fn test_glasscore_v3() {
		let mut features = HashMap::<String, HashMap<String, String>>::new();
		features.insert("mineshaft".to_string(), { let mut map = HashMap::new(); map.insert("chance".to_string(), "0.04".to_string()); map });
		
		let mut default = FlatV3 {
			layers: vec![
				LayerV3 { count: 1, namespace: "minecraft".to_string(), id: "bedrock".to_string(), meta: 0 },
				LayerV3 { count: 63, namespace: "minecraft".to_string(), id: "glass".to_string(), meta: 0 },
			],
			biome: 1,
			features
		};
		
		default.add_feature("dungeon");
		default.add_feature("decoration");
		default.add_feature("lake");
		default.add_feature("lava_lake");
		default.add_feature("stronghold");
		
		assert_eq!(Ok(default), "3;1*minecraft:bedrock,63*minecraft:glass;1;mineshaft(chance=0.04),dungeon,decoration,lake,lava_lake,stronghold".parse::<FlatV3>())
	}
}