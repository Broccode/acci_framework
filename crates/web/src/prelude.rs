// crates/web/src/prelude.rs
// Zentrale Importdatei für alle gemeinsam genutzten Traits und Typen

// Importiere die Standard-Traits von Leptos
pub use leptos::html::ElementChild;

// Importiere unsere eigenen Leptos-Implementierungen
pub use crate::services::leptos::{
    Scope,
    LeptosOptions,
    ssr,
    HtmlElement,
    TextNode,
    Node,
    IntoView,
    Fragment,
};

// Für die Verwendung in Komponenten, müssen die Makros direkt importiert werden:
// use crate::component;
// use crate::view;