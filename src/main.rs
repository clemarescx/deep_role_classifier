mod classifier;
use classifier::{ClassificationMethod::VectorProjection, Classifier, ModelFormat};

const _DEEP_ROLES_CSV: &str = "assets/archetypes.csv";
const DEEP_ROLES_JSON: &str = "assets/archetypes.json";
const EXAMPLE_CSV: &str = "assets/profiles/example.csv";
fn main() {
    let classifier = Classifier::new(ModelFormat::Json(DEEP_ROLES_JSON));
    let profiles_to_classify = classifier::load_deep_role_profiles(ModelFormat::Csv(EXAMPLE_CSV));

    for profile in profiles_to_classify {
        let method = VectorProjection;
        let mut ranking = classifier.classify_profile(&profile, method);
        println!("Ranking of archetypes for profile '{}':", profile.name);
        let mut i = 1;
        while let Some(ranking) = ranking.pop() {
            println!("{}. {} - {:3.2}%", i, ranking.name, ranking.rank * 100.);
            i += 1;
        }

        println!();
    }
}
