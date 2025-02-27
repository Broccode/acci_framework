// crates/web/src/prelude.rs
// Zentrale Importdatei für alle gemeinsam genutzten Traits und Typen

// Importiere die Standard-Traits von Leptos
pub use leptos::html::ElementChild;

// Importiere unsere eigenen Leptos-Implementierungen
pub use crate::services::leptos::{
    Fragment, HtmlElement, IntoView, LeptosOptions, Node, Scope, TextNode, ssr,
};

// Für die Verwendung in Komponenten, müssen die Makros direkt importiert werden:
// use crate::component;
// use crate::view;
