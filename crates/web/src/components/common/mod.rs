// Common Components Module
// Allgemeine Komponenten, die in mehreren Bereichen wiederverwendet werden

pub mod error_display;
pub mod loading_indicator;

// Re-exports für häufig verwendete Komponenten
pub use error_display::*;
pub use loading_indicator::*; 