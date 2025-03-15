use leptos::prelude::*;

#[cfg(feature = "enable_webauthn")]
use crate::services::auth::AuthService;

// Conditional imports for WebAuthn
#[cfg(feature = "enable_webauthn")]
use js_sys;
#[cfg(feature = "enable_webauthn")]
use uuid::Uuid;
#[cfg(feature = "enable_webauthn")]
use wasm_bindgen::prelude::*;

// JavaScript bindings for WebAuthn operations
#[cfg(feature = "enable_webauthn")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = window, js_name = "webauthn.startRegistration")]
    fn startRegistration(options: JsValue) -> js_sys::Promise;

    #[wasm_bindgen(js_namespace = window, js_name = "webauthn.startAuthentication")]
    fn startAuthentication(options: JsValue) -> js_sys::Promise;
}

/// WebAuthn registration form component
#[component]
#[cfg(feature = "enable_webauthn")]
pub fn WebAuthnRegistrationForm(
    #[prop(into)] user_id: Signal<Uuid>,
    #[prop(optional)] credential_name: Option<Signal<String>>,
    #[prop(optional)] _on_success: Option<Callback<String>>,
    #[prop(optional)] _on_error: Option<Callback<String>>,
    #[prop(optional)] _on_cancel: Option<Callback<()>>,
) -> impl IntoView {
    // Get authentication service from context
    let auth_service = use_context::<AuthService>().expect("AuthService must be provided");

    // Component state
    let error = RwSignal::new(None::<String>);
    let loading = RwSignal::new(false);
    let name = credential_name.unwrap_or_else(|| RwSignal::new("Security Key".to_string()).into());

    // Handler for registration button click
    let register = move |_| {
        // Stub implementation that doesn't perform actual WebAuthn operations
        // This will compile but won't do anything
        error.set(Some(
            "WebAuthn operations disabled for this build.".to_string(),
        ));
    };

    // Handle cancel button - just a no-op for now
    let handle_cancel = move |_| {};

    // Component view
    view! {
        <div class="webauthn-registration-form card p-4">
            <h3 class="card-title text-center">"Register Security Key"</h3>
            <p class="card-subtitle text-center text-muted mb-4">
                "Use a hardware security key, fingerprint, or facial recognition
                 to secure your account with strong authentication."
            </p>

            <div class="mb-3">
                <label for="credential-name" class="form-label">"Key Name"</label>
                <input
                    type="text"
                    id="credential-name"
                    class="form-control"
                    prop:value=move || name.get()
                    on:input=move |_ev| {
                        // Input handling disabled for simplified implementation
                    }
                />
                <div class="form-text">"Give this security key a name to identify it"</div>
            </div>

            <div class="error-container">
                {
                    move || error.get().map(|err| {
                        view! {
                            <div class="alert alert-danger">
                                {err.clone()}
                            </div>
                        }
                    })
                }
            </div>

            <Show
                when=move || loading.get()
                fallback=move || {
                    view! {
                        <div class="d-grid gap-2">
                            <button
                                type="button"
                                class="btn btn-primary"
                                on:click=move |_| { register(()); }
                            >
                                "Register Security Key"
                            </button>
                            <button
                                type="button"
                                class="btn btn-outline-secondary"
                                on:click=handle_cancel
                            >
                                "Cancel"
                            </button>
                        </div>
                    }
                }
            >
                <div class="text-center">
                    <div class="spinner-border" role="status">
                        <span class="visually-hidden">"Loading..."</span>
                    </div>
                    <p>"Waiting for your security key..."</p>
                </div>
            </Show>
        </div>
    }
}

/// WebAuthn authentication form component - fallback for when WebAuthn is disabled
#[component]
#[cfg(not(feature = "enable_webauthn"))]
pub fn WebAuthnRegistrationForm(
    #[prop(into)] _user_id: Signal<String>,
    #[prop(optional)] _credential_name: Option<Signal<String>>,
    #[prop(optional)] _on_success: Option<Callback<String>>,
    #[prop(optional)] _on_error: Option<Callback<String>>,
    #[prop(optional)] _on_cancel: Option<Callback<()>>,
) -> impl IntoView {
    view! {
        <div class="webauthn-disabled">
            <p>"WebAuthn is not available in this build."</p>
        </div>
    }
}

/// WebAuthn authentication form component - fallback for when WebAuthn is disabled
#[component]
#[cfg(not(feature = "enable_webauthn"))]
pub fn WebAuthnAuthenticationForm(
    #[prop(optional)] _user_id: Option<Signal<String>>,
    #[prop(into)] _session_id: Signal<String>,
    #[prop(optional)] _on_success: Option<Callback<()>>,
    #[prop(optional)] _on_error: Option<Callback<String>>,
    #[prop(optional)] _on_cancel: Option<Callback<()>>,
) -> impl IntoView {
    view! {
        <div class="webauthn-disabled">
            <p>"WebAuthn is not available in this build."</p>
        </div>
    }
}

/// WebAuthn authentication form component
#[component]
#[cfg(feature = "enable_webauthn")]
pub fn WebAuthnAuthenticationForm(
    #[prop(optional)] user_id: Option<Signal<Uuid>>,
    #[prop(into)] session_id: Signal<Uuid>,
    #[prop(optional)] _on_success: Option<Callback<()>>,
    #[prop(optional)] _on_error: Option<Callback<String>>,
    #[prop(optional)] _on_cancel: Option<Callback<()>>,
) -> impl IntoView {
    // Get authentication service from context
    let auth_service = use_context::<AuthService>().expect("AuthService must be provided");

    // Component state
    let error = RwSignal::new(None::<String>);
    let loading = RwSignal::new(false);

    // Handler for authentication button click
    let authenticate = move |_| {
        // Stub implementation that doesn't perform actual WebAuthn operations
        // This will compile but won't do anything
        error.set(Some(
            "WebAuthn authentication disabled for this build.".to_string(),
        ));
    };

    // Handle cancel button
    let handle_cancel = move |_| {};

    // Component view
    view! {
        <div class="webauthn-authentication-form card p-4">
            <h3 class="card-title text-center">"Use Security Key"</h3>
            <p class="card-subtitle text-center text-muted mb-4">
                "Authenticate with your security key, fingerprint, or facial recognition."
            </p>

            <div class="error-container">
                {
                    move || error.get().map(|err| {
                        view! {
                            <div class="alert alert-danger">
                                {err.clone()}
                            </div>
                        }
                    })
                }
            </div>

            <Show
                when=move || loading.get()
                fallback=move || {
                    view! {
                        <div class="d-grid gap-2">
                            <button
                                type="button"
                                class="btn btn-primary"
                                on:click=move |_| { authenticate(()); }
                            >
                                "Authenticate with Security Key"
                            </button>
                            <button
                                type="button"
                                class="btn btn-outline-secondary"
                                on:click=handle_cancel
                            >
                                "Cancel"
                            </button>
                        </div>
                    }
                }
            >
                <div class="text-center">
                    <div class="spinner-border" role="status">
                        <span class="visually-hidden">"Loading..."</span>
                    </div>
                    <p>"Waiting for your security key..."</p>
                </div>
            </Show>
        </div>
    }
}
