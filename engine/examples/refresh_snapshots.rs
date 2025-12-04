use engine::sim::test_fixtures;

fn main() {
    for fixture in test_fixtures::all() {
        let result = fixture
            .run()
            .unwrap_or_else(|err| panic!("{} scenario failed: {err}", fixture.name));
        let path = format!(
            "{}/src/sim/test_fixtures/{}_snapshot.json",
            env!("CARGO_MANIFEST_DIR"),
            fixture.name
        );
        let json = serde_json::to_string_pretty(&result)
            .unwrap_or_else(|err| panic!("failed to serialize {}: {err}", fixture.name));
        std::fs::write(&path, json).expect("failed to write snapshot file");
        println!("updated {path}");
    }
}
