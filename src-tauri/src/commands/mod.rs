// Command modules
pub mod game_library;
pub mod bangumi;
pub mod settings;
pub mod tags;

// Re-export all commands for easy access
pub use game_library::*;
pub use bangumi::*;
pub use settings::*;
pub use tags::*;
