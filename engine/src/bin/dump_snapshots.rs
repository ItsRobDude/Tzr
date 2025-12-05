use engine::sim::test_fixtures;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    for fixture in test_fixtures::all() {
        let result = fixture
            .run()
            .unwrap_or_else(|err| panic!("{} scenario failed: {err}", fixture.name));

        println!("=== {} ===", fixture.name);
        let json =
            serde_json::to_string_pretty(&result).expect("failed to serialize SimulationResult");
        println!("{json}\n");
    }

    Ok(())
}
