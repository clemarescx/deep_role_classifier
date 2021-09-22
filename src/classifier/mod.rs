pub mod models;

use csv::Reader;
use models::{DeepRole, NAME_KEY};
use std::cmp::{Ord, Ordering};
use std::{
    collections::{BinaryHeap, HashMap},
    io::prelude::*,
};

#[allow(dead_code)]
pub enum ClassificationMethod {
    AngularDistance,
    VectorProjection,
}
pub enum ModelFormat<'a> {
    Csv(&'a str),
    Json(&'a str),
}
pub struct Classifier {
    pub archetypes: Vec<DeepRole>,
}

pub struct Rank {
    pub name: String,
    pub rank: f32,
}

impl Eq for Rank {}

impl PartialEq for Rank {
    fn eq(&self, other: &Rank) -> bool {
        self.rank == other.rank && self.name == other.name
    }
}

impl PartialOrd for Rank {
    fn partial_cmp(&self, other: &Rank) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Rank {
    fn cmp(&self, other: &Self) -> Ordering {
        self.rank.partial_cmp(&other.rank).unwrap()
    }
}

pub fn load_deep_role_profiles(file_path: ModelFormat) -> Vec<DeepRole> {
    let mut profiles = match file_path {
        ModelFormat::Csv(fp) => load_deeproles_csv(fp),
        ModelFormat::Json(fp) => load_deeproles_json(fp),
    };
    for p in profiles.iter_mut() {
        p.normalize_facet_values()
    }
    profiles
}

impl Classifier {
    pub fn new(archetypes_file_path: ModelFormat) -> Self {
        Classifier {
            archetypes: load_deep_role_profiles(archetypes_file_path),
        }
    }

    pub fn classify_profile(
        &self,
        profile: &DeepRole,
        method: ClassificationMethod,
    ) -> BinaryHeap<Rank> {
        let get_rank = match method {
            ClassificationMethod::AngularDistance => angular_distance,
            ClassificationMethod::VectorProjection => vector_projection,
        };

        let mut ranking = BinaryHeap::new();
        for archetype in self.archetypes.iter() {
            let rank = get_rank(profile, archetype);
            ranking.push(rank);
        }
        ranking
    }
}
fn vector_projection(profile: &DeepRole, archetype: &DeepRole) -> Rank {
    let scale = profile.dot(archetype);
    Rank {
        name: archetype.name.clone(),
        rank: scale,
    }
}

fn angular_distance(profile: &DeepRole, archetype: &DeepRole) -> Rank {
    let angle = archetype.angle(profile);
    Rank {
        name: archetype.name.clone(),
        rank: 1. / angle,
    }
}

pub fn load_deeproles_json(file_path: &str) -> Vec<DeepRole> {
    let mut archetype_json = match std::fs::File::open(file_path) {
        Ok(fp) => fp,
        Err(e) => panic!("could not open file {}: {}", file_path, e),
    };

    let mut file_content = String::new();
    archetype_json
        .read_to_string(&mut file_content)
        .expect("cannot read file");
    let archetypes: Vec<DeepRole> =
        serde_json::from_str(&file_content).expect("cannot deserialize");
    archetypes
}

pub fn load_deeproles_csv(file_path: &str) -> Vec<DeepRole> {
    let records_to_hashmap = parse_to_hashmap(file_path);
    prime_deep_roles(records_to_hashmap)
}

pub fn parse_to_hashmap(file_path: &str) -> Vec<HashMap<String, String>> {
    let deep_role_csv = std::fs::File::open(file_path)
        .expect("the archetypes.csv model should be in the same folder as the executable");
    let file_reader = std::io::BufReader::new(deep_role_csv);
    let mut csv_reader = Reader::from_reader(file_reader);
    let mut hashmaps: Vec<HashMap<String, String>> = vec![];
    for r in csv_reader.deserialize() {
        let record: HashMap<String, String> = r.expect("error in archetypes csv format");
        hashmaps.push(record);
    }
    hashmaps
}

fn prime_deep_roles(records: Vec<HashMap<String, String>>) -> Vec<DeepRole> {
    let mut deep_roles: Vec<DeepRole> = vec![];
    for mut record in records {
        let name = record.remove(NAME_KEY).expect("missing name value");
        let facets = record
            .into_iter()
            .map(|(k, v)| {
                let parsed = match v.parse() {
                    Ok(val) => val,
                    Err(_) => panic!("could not parse value {} for archetype {}", k, name),
                };
                (k, parsed)
            })
            .collect();
        let arch = DeepRole { name, facets };
        deep_roles.push(arch);
    }
    deep_roles
}

#[allow(dead_code)]
pub fn hashmap_to_json(records: &[HashMap<String, String>]) -> String {
    let mut json: Vec<String> = vec![];
    for record in records {
        let name = &record["name"];
        let mut facets: Vec<String> = vec![];
        for (attribute, value) in record.iter().filter(|&(k, _)| k != "name") {
            if let Ok(val) = value.parse::<i16>() {
                facets.push(format!("\"{}\":{}", attribute, val));
            }
        }
        let arch_object = facets.join(",");
        json.push(format!(
            "{{ \"name\": \"{}\", \"facets\":{{{}}}}}",
            name, arch_object
        ));
    }
    let inner = json.join(",");
    format!("[{}]", inner)
}
