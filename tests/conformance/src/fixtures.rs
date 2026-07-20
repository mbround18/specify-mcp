#[derive(Debug, Clone)]
pub struct FixtureTaskGraph {
    pub parent: String,
    pub tasks: Vec<String>,
}

pub fn parent_plus_twenty_fixture() -> FixtureTaskGraph {
    FixtureTaskGraph {
        parent: "PARENT".to_string(),
        tasks: (1..=20).map(|n| format!("T{n:03}")).collect(),
    }
}
