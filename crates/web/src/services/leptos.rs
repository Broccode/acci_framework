/// LeptosOptions enthält Konfigurationsoptionen für den Leptos-SSR-Renderer
#[derive(Clone)]
pub struct LeptosOptions {
    /// Der Pfad, unter dem die statischen Assets zu finden sind
    pub site_root: String,
    /// Die Site-Präfix (z.B. "/app" für eine Anwendung unter example.com/app)
    pub site_pkg_dir: String,
    /// Der Titel der Anwendung
    pub site_name: String,
    /// Flag, ob SSR aktiviert ist
    pub ssr_enabled: bool,
}

impl Default for LeptosOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl LeptosOptions {
    /// Erstellt eine neue Standard-Instanz der LeptosOptions
    pub fn new() -> Self {
        Self {
            site_root: "static".into(),
            site_pkg_dir: "pkg".into(),
            site_name: "ACCI Framework".into(),
            ssr_enabled: true,
        }
    }
}

/// Ein einfacher Scope für Leptos-Komponenten
/// In einer echten Implementierung wäre dies ein komplexerer Typ
#[derive(Clone, Copy)]
pub struct Scope;

impl Scope {
    pub fn create_node(&self, _tag: &str) -> HtmlElement {
        HtmlElement {}
    }

    pub fn create_text_node(&self, _text: &str) -> TextNode {
        TextNode {}
    }
}

/// HTML-Element für unsere einfache Leptos-Implementierung
#[derive(Clone)]
pub struct HtmlElement;

impl HtmlElement {
    pub fn attr(&self, _name: &str, _value: impl AsRef<str>) -> &Self {
        self
    }

    pub fn child(&self, _node: impl Into<Node>) -> &Self {
        self
    }

    pub fn into_view(self) -> Node {
        Node::Element(self)
    }
}

/// Text-Node für unsere einfache Leptos-Implementierung
#[derive(Clone)]
pub struct TextNode;

impl TextNode {
    pub fn into_view(self) -> Node {
        Node::Text(self)
    }
}

/// Node-Enum für unsere einfache Leptos-Implementierung
#[derive(Clone)]
pub enum Node {
    Element(HtmlElement),
    Text(TextNode),
    Fragment(Vec<Node>),
}

/// Ein Trait für Typen, die in eine View umgewandelt werden können
pub trait IntoView {
    fn into_view(self) -> Node;
    fn to_string(&self) -> String;
}

impl IntoView for String {
    fn into_view(self) -> Node {
        Node::Text(TextNode {})
    }

    fn to_string(&self) -> String {
        self.clone()
    }
}

impl IntoView for &'static str {
    fn into_view(self) -> Node {
        Node::Text(TextNode {})
    }

    fn to_string(&self) -> String {
        (*self).to_string()
    }
}

impl IntoView for Node {
    fn into_view(self) -> Node {
        self
    }

    fn to_string(&self) -> String {
        match self {
            Node::Element(_) => "<element>...</element>".to_string(),
            Node::Text(_) => "text".to_string(),
            Node::Fragment(_) => "<fragment>...</fragment>".to_string(),
        }
    }
}

/// Fragment-Typ für unsere einfache Leptos-Implementierung
pub struct Fragment(Vec<Node>);

impl Default for Fragment {
    fn default() -> Self {
        Self::new()
    }
}

impl Fragment {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn child(mut self, node: impl IntoView) -> Self {
        self.0.push(node.into_view());
        self
    }

    pub fn into_view(self) -> Node {
        Node::Fragment(self.0)
    }
}

/// Diese Module implementieren die Funktionen für das serverseitige Rendering (SSR)
pub mod ssr {
    use super::*;

    /// Rendert eine View-Funktion zu einem HTML-String mit dem gegebenen Kontext
    pub fn render_to_string_with_context<F, V>(_options: &LeptosOptions, view_fn: F) -> String
    where
        F: FnOnce(Scope) -> V,
        V: IntoView,
    {
        // In einer realen Implementierung würde hier der Leptos-Renderer verwendet werden
        // Für unsere Demonstrationszwecke geben wir einfach den String zurück
        let scope = Scope;
        let view = view_fn(scope);

        // In einer realen Implementierung würden hier noch zusätzliche Header und Metadaten hinzugefügt werden
        view.to_string()
    }
}

/// Makro für die View-Definition
/// In einer realen Implementierung wäre dies ein echtes Proc-Makro
#[macro_export]
macro_rules! view {
    ($cx:expr, $($rest:tt)*) => {{
        let scope: $crate::services::leptos::Scope = $cx;

        // Dies ist nur eine vereinfachte Implementierung
        // In einer realen Implementierung würde hier DOM-Elemente erzeugt werden
        let fragment = $crate::services::leptos::Fragment::new();

        // Dummy-Implementierung, die nur einen String zurückgibt
        fragment.into_view()
    }};
}

/// Komponenten-Annotation
/// In einer realen Implementierung wäre dies ein echtes Proc-Makro
#[macro_export]
macro_rules! component {
    ($(#[$attr:meta])* $vis:vis fn $name:ident($($args:tt)*) -> impl IntoView $body:block) => {
        $(#[$attr])*
        $vis fn $name($($args)*) -> impl $crate::services::leptos::IntoView $body
    };
}
