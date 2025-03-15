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
// Explicitly export modules to avoid ambiguous reexports
pub use prelude::*;
pub use routes::create_router;

// Export specific components with disambiguated names
pub use components::auth::login_form::login_form_ssr;
pub use components::auth::registration_form::registration_form_ssr;
pub use components::auth::verification_form::{send_verification_form_ssr, verification_form_ssr};
pub use components::common::error_display::error_display_ssr;
pub use components::common::loading_indicator::loading_indicator_ssr;
pub use components::layout::footer::footer_ssr;
pub use components::layout::navigation::navigation_ssr;

// Export page rendering functions
pub use pages::home::render_home_page;
pub use pages::login::render_login_page;
pub use pages::register::render_register_page;
pub use pages::verify::{render_send_verify_page, render_verify_page};
