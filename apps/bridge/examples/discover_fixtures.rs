//! Print DiscoverResult JSON against packaged fixtures (default: no files[]).
use std::path::PathBuf;
use tokentracer_bridge::discover_fixtures;

fn main() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let result = discover_fixtures(&root);
    println!("{}", serde_json::to_string_pretty(&result).expect("json"));
    let agents: std::collections::BTreeSet<_> =
        result.sources.iter().map(|s| s.agent.as_str()).collect();
    eprintln!(
        "sources={}, errors={}, agents={:?}",
        result.sources.len(),
        result.errors.len(),
        agents
    );
}
