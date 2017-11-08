use structure::organized::BoundingBox;
use std::str::FromStr;
use std::fmt;
use std::collections::HashMap;
use serde::{Serialize, Serializer, Deserialize, Deserializer};
use serde::de::{self, Visitor};

#[derive(Serialize, Deserialize, Debug)]
pub struct VillageDatabase {
	pub data: VillageData
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VillageData {
	#[serde(rename="Features")]
	pub features: HashMap<String, VillageStructure>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VillageStructure {
	                            pub id: String,
	#[serde(rename="ChunkX")]   pub chunk_x: i32,
	#[serde(rename="ChunkZ")]   pub chunk_z: i32,
	#[serde(rename="BB")]       pub bounding_box: BoundingBox,
	#[serde(rename="Valid")]    pub valid: bool,
	#[serde(rename="Children")] pub children: Vec<VillagePiece>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VillagePiece {
	#[serde(rename="id")]     kind: PieceKind,
	#[serde(rename="GD")]     gd: i32,
	#[serde(rename="O")]      orientation: i32,
	#[serde(rename="BB")]     bounding_box: BoundingBox,
	
	/// "Style" of the village. 0=Plains, 1=Desert, 2=Savanna, 3=Taiga. Not present in older versions.
	#[serde(rename="Type")]   style: Option<i8>,
	/// Is this village a zombvie village instead of normal village? Not present in older versions.
	#[serde(rename="Zombie")] zombie: Option<bool>,
	/// Initial villagers this piece spawned with.
	#[serde(rename="VCount")] initial_villagers: i32,
	/// Y value the terrain will be raised/lowered to for placing this structure. -1 if not set.
	#[serde(rename="HPos")]   y_base: i32,
	
	/// [Blacksmith]
	#[serde(rename="Chest")] chest:   Option<bool>,
	/// [Farm/FarmDouble]
	/// ID of the first crop.
	#[serde(rename="CA")] crop_a:  Option<i32>,
	/// [Farm/FarmDouble]
	/// ID of the second crop.
	#[serde(rename="CB")] crop_b:  Option<i32>,
	/// [FarmDouble]
	/// ID of the third crop.
	#[serde(rename="CC")] crop_c:  Option<i32>,
	/// [FarmDouble]
	/// ID of the fourth crop.
	#[serde(rename="CD")] crop_d:  Option<i32>,
	/// [HouseSmall]
	/// Has the outer dirt "terrace" been generated?.
	#[serde(rename="Terrace")] terrace: Option<bool>,
	/// [Hut]
	/// 0 = No Table, 1/2 = Table
	#[serde(rename="T")] table:   Option<i32>,
	/// [Hut]
	#[serde(rename="C")] roof:    Option<bool>,
	/// [Path]
	/// Length in blocks of the road.
	#[serde(rename="Length")] length:  Option<i32>
}

#[derive(Debug)]
pub enum PieceKind {
	Start,
	Library,
	Farm,
	FarmDouble,
	HouseSmall,
	HouseLarge,
	Hut,
	Well,
	Path,
	Blacksmith,
	Church,
	Butcher,
	Light
}

impl PieceKind {
	fn id(&self) -> &'static str {
		match *self {
			PieceKind::Start      => "ViStart",
			PieceKind::Library    => "ViBH",
			PieceKind::Farm       => "ViF",
			PieceKind::FarmDouble => "ViDF",
			PieceKind::HouseSmall => "ViSH",
			PieceKind::HouseLarge => "ViTRH",
			PieceKind::Hut        => "ViSmH",
			PieceKind::Well       => "ViW",
			PieceKind::Path       => "ViSR",
			PieceKind::Blacksmith => "ViS",
			PieceKind::Church     => "ViST",
			PieceKind::Butcher    => "ViPH",
			PieceKind::Light      => "ViL"
		}
	}
}

impl FromStr for PieceKind {
	type Err = ();
	
	fn from_str(s: &str) -> Result<Self, ()> {
		Ok(match s {
			"ViStart" => PieceKind::Start,
			"ViBH"    => PieceKind::Library,
			"ViF"     => PieceKind::Farm,
			"ViDF"    => PieceKind::FarmDouble,
			"ViSH"    => PieceKind::HouseSmall,
			"ViTRH"    => PieceKind::HouseLarge,
			"ViSmH"   => PieceKind::Hut,
			"ViW"     => PieceKind::Well,
			"ViSR"    => PieceKind::Path,
			"ViS"     => PieceKind::Blacksmith,
			"ViST"    => PieceKind::Church,
			"ViPH"    => PieceKind::Butcher,
			"ViL"     => PieceKind::Light,
			_         => return Err(())
		})
	}
}

impl Serialize for PieceKind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        serializer.serialize_str(self.id())
    }
}

impl Deserialize for PieceKind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer
    {
        deserializer.deserialize_str(PieceKindVisitor)
    }
}

const PIECES: &[&str] = &["ViStart", "ViBH", "ViF", "ViDF", "ViSH", "ViTRH", "ViSmH", "ViW", "ViSR", "ViS", "ViST", "ViPH", "ViL"];

struct PieceKindVisitor;

impl Visitor for PieceKindVisitor {
    type Value = PieceKind;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a Village piece identifier starting with 'Vi'")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where E: de::Error
    {
        value.parse::<PieceKind>().map_err(|_| E::unknown_variant(value, PIECES))
    }
}
