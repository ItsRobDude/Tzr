#[derive(Debug, thiserror::Error)]
pub enum SimError {
    #[error("Entity limit exceeded")]
    EntityLimit,
    #[error("Event limit exceeded")]
    EventLimit,
    #[error("Tick limit exceeded")]
    TickLimit,
    #[error("Invalid dungeon: {0}")]
    InvalidDungeon(String),
}
