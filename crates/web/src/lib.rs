// acci_web crate - Leptos SSR Implementation
// Die gesamte Web-Implementierung erfolgt ausschließlich im SSR-Modus ohne WebAssembly

#[macro_use]
pub mod services;

pub mod components;
pub mod pages;
pub mod handlers;
pub mod routes;
pub mod utils;
pub mod prelude;

// Re-exports für häufig verwendete Typen und Funktionen
pub use components::*;
pub use pages::*;
pub use routes::*;
pub use services::*;
pub use prelude::*;
