use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const NAME_KEY: &str = "name";

#[allow(dead_code)]
pub struct PersonalTrait {
    pub name: String,
    pole: Pole,
}

#[allow(dead_code)]
pub struct Pole {
    pub negative: [(String, f32); 6],
    pub positive: [(String, f32); 6],
}

#[derive(Serialize, Deserialize)]
pub struct DeepRole {
    pub name: String,
    pub facets: HashMap<String, f32>,
}

impl DeepRole {
    pub fn cos_theta(&self, other: &DeepRole) -> f32 {
        let dot: f32 = self.dot(other);
        let magnitude_self = self.magnitude();
        let magnitude_other = other.magnitude();
        let cos_theta: f32 = dot / (magnitude_self * magnitude_other);
        cos_theta
    }

    pub fn angle(&self, other: &DeepRole) -> f32 {
        let cos_theta = self.cos_theta(other);
        match cos_theta {
            x if x > 1.0 => 1f32.acos(),
            x if x < -1.0 => (-1f32).acos(),
            _ => cos_theta.acos(),
        }
    }

    pub fn dot(&self, other: &Self) -> f32 {
        let other_facets = &other.facets;
        let dot: f32 = self.facets.iter().map(|(k, v)| v * other_facets[k]).sum();
        dot
    }

    pub fn magnitude(&self) -> f32 {
        let mag_squared: f32 = self.facets.values().map(|v| v * v).sum();
        mag_squared.sqrt()
    }

    pub fn normalize_facet_values(&mut self) {
        let mag = self.magnitude();
        if mag > 0.0 {
            for v in self.facets.values_mut() {
                *v /= mag;
            }
        } else {
            panic!("vector of {} has a magnitude of 0", self.name)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::DeepRole;
    use crate::{
        classifier::{Classifier, ModelFormat},
        DEEP_ROLES_JSON,
    };
    use std::collections::HashMap;

    fn rad_to_deg(rad: f32) -> f32 {
        rad * 180. / std::f32::consts::PI
    }

    fn into_hashmap(kvps: &[(&str, f32)]) -> HashMap<String, f32> {
        kvps.iter().fold(HashMap::new(), |mut acc, (k, v)| {
            acc.insert(k.to_string(), *v);
            acc
        })
    }

    #[test]
    fn angle_with_self_is_zero() {
        let classifier = Classifier::new(ModelFormat::Json(DEEP_ROLES_JSON));
        let arch = &classifier.archetypes[0];
        let angle = arch.angle(arch);
        assert!(angle <= f32::EPSILON);
    }

    #[test]
    fn angle_with_self_is_zero_2() {
        let arch = {
            DeepRole {
                name: "test".to_string(),
                facets: into_hashmap(&[("x", 1f32), ("y", 1f32)]),
            }
        };
        let angle = arch.angle(&arch);
        assert!(angle <= f32::EPSILON)
    }
    #[test]
    fn assert_45deg_angle() {
        let a = DeepRole {
            name: "test".to_string(),
            facets: into_hashmap(&[("x", 1f32), ("y", 1f32)]),
        };
        let b = DeepRole {
            name: "test".to_string(),
            facets: into_hashmap(&[("x", 1f32), ("y", 0f32)]),
        };
        let angle = a.angle(&b);
        let angle_deg = rad_to_deg(angle);
        assert!((45.0 - angle_deg).abs() <= f32::EPSILON)
    }

    #[test]
    fn dot_with_self_equals_magnitude_squared() {
        let classifier = Classifier::new(ModelFormat::Json(DEEP_ROLES_JSON));
        let arch = &classifier.archetypes[0];
        let dot = arch.dot(arch);
        let mag = arch.magnitude();
        let mag_squared = mag * mag;
        assert!((mag_squared - dot).abs() <= f32::EPSILON);
    }
}
