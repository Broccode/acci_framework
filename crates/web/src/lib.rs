// acci_web crate - Leptos SSR Implementation
// Die gesamte Web-Implementierung erfolgt ausschließlich im SSR-Modus ohne WebAssembly

#[macro_use]
pub mod services;

pub mod components;
pub mod handlers;
pub mod pages;
pub mod prelude;
pub mod routes;
pub mod utils;

// Re-exports für häufig verwendete Typen und Funktionen
pub use components::*;
pub use pages::*;
pub use prelude::*;
pub use routes::*;
pub use services::*;
