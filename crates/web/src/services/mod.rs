// Services Module
// Dieses Modul enthält die Serviceschicht für die Web-Anwendung

pub mod auth;
pub mod leptos;

// Re-exports für häufig verwendete Services
pub use auth::*;
pub use leptos::*;
