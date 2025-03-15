use crate::{
    components::common::{error_display::ErrorDisplay, loading_indicator::LoadingIndicator},
    services::auth::AuthService,
};
use leptos::*;
use uuid::Uuid;
use wasm_bindgen::prelude::*;

// JavaScript bindings for WebAuthn operations
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = window.webauthn)]
    fn startRegistration(options: JsValue) -> js_sys::Promise;

    #[wasm_bindgen(js_namespace = window.webauthn)]
    fn startAuthentication(options: JsValue) -> js_sys::Promise;
}

/// WebAuthn registration form component
#[component]
pub fn WebAuthnRegistrationForm(
    cx: Scope,
    #[prop(into)] user_id: MaybeSignal<Uuid>,
    #[prop(optional)] credential_name: Option<MaybeSignal<String>>,
    #[prop(optional)] on_success: Option<Callback<String>>,
    #[prop(optional)] on_error: Option<Callback<String>>,
    #[prop(optional)] on_cancel: Option<Callback<()>>,
) -> impl IntoView {
    // Get authentication service from context
    let auth_service = use_context::<AuthService>(cx).expect("AuthService must be provided");

    // Component state
    let (error, set_error) = create_signal(cx, None::<String>);
    let (loading, set_loading) = create_signal(cx, false);
    let name =
        credential_name.unwrap_or_else(|| create_signal(cx, "Security Key".to_string()).0.into());

    // Handler for registration button click
    let register = create_action(cx, move |_| {
        let user_id = user_id.get();
        let auth_service = auth_service.clone();
        let name = name.get();
        let set_error = set_error.clone();
        let set_loading = set_loading.clone();
        let on_success = on_success.clone();
        let on_error = on_error.clone();

        set_loading.set(true);
        set_error.set(None);

        async move {
            // Step 1: Get registration options from server
            match auth_service.start_webauthn_registration(user_id).await {
                Ok(options) => {
                    // Step 2: Pass options to browser's WebAuthn API
                    let js_options =
                        serde_wasm_bindgen::to_value(&options).unwrap_or_else(|_| JsValue::NULL);

                    match wasm_bindgen_futures::JsFuture::from(startRegistration(js_options)).await
                    {
                        Ok(credential) => {
                            // Step 3: Send credential to server for verification
                            match auth_service
                                .finish_webauthn_registration(
                                    user_id,
                                    &name,
                                    serde_wasm_bindgen::from_value(credential).unwrap(),
                                )
                                .await
                            {
                                Ok(credential_id) => {
                                    set_loading.set(false);
                                    if let Some(on_success) = on_success {
                                        on_success.call(credential_id);
                                    }
                                },
                                Err(e) => {
                                    set_loading.set(false);
                                    set_error
                                        .set(Some(format!("Server verification failed: {}", e)));
                                    if let Some(on_error) = on_error {
                                        on_error.call(e);
                                    }
                                },
                            }
                        },
                        Err(e) => {
                            set_loading.set(false);
                            let error_msg = e
                                .as_string()
                                .unwrap_or_else(|| "Unknown browser error".to_string());
                            set_error.set(Some(error_msg.clone()));
                            if let Some(on_error) = on_error {
                                on_error.call(error_msg);
                            }
                        },
                    }
                },
                Err(e) => {
                    set_loading.set(false);
                    set_error.set(Some(format!("Failed to start registration: {}", e)));
                    if let Some(on_error) = on_error {
                        on_error.call(e);
                    }
                },
            }
        }
    });

    // Handle cancel button
    let handle_cancel = move |_| {
        if let Some(on_cancel) = on_cancel {
            on_cancel.call(());
        }
    };

    // Component view
    view! { cx,
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
                    on:input=move |ev| {
                        if let Some(credential_name) = credential_name {
                            if let Some(input) = event_target_value(&ev) {
                                if let Some(signal) = credential_name.as_signal() {
                                    signal.set(input);
                                }
                            }
                        }
                    }
                />
                <div class="form-text">"Give this security key a name to identify it"</div>
            </div>

            <ErrorDisplay error=error />

            <Show
                when=move || loading.get()
                fallback=move |cx| view! { cx,
                    <div class="d-grid gap-2">
                        <button
                            type="button"
                            class="btn btn-primary"
                            on:click=move |_| register.dispatch(())
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
            >
                <div class="text-center">
                    <LoadingIndicator />
                    <p>"Waiting for your security key..."</p>
                </div>
            </Show>
        </div>
    }
}

/// WebAuthn authentication form component
#[component]
pub fn WebAuthnAuthenticationForm(
    cx: Scope,
    #[prop(optional)] user_id: Option<MaybeSignal<Uuid>>,
    #[prop(into)] session_id: MaybeSignal<Uuid>,
    #[prop(optional)] on_success: Option<Callback<()>>,
    #[prop(optional)] on_error: Option<Callback<String>>,
    #[prop(optional)] on_cancel: Option<Callback<()>>,
) -> impl IntoView {
    // Get authentication service from context
    let auth_service = use_context::<AuthService>(cx).expect("AuthService must be provided");

    // Component state
    let (error, set_error) = create_signal(cx, None::<String>);
    let (loading, set_loading) = create_signal(cx, false);

    // Handler for authentication button click
    let authenticate = create_action(cx, move |_| {
        let auth_service = auth_service.clone();
        let user_id_val = user_id.clone().map(|s| s.get());
        let session_id = session_id.get();
        let set_error = set_error.clone();
        let set_loading = set_loading.clone();
        let on_success = on_success.clone();
        let on_error = on_error.clone();

        set_loading.set(true);
        set_error.set(None);

        async move {
            // Step 1: Get authentication options from server
            match auth_service
                .start_webauthn_authentication(user_id_val)
                .await
            {
                Ok(options) => {
                    // Step 2: Pass options to browser's WebAuthn API
                    let js_options =
                        serde_wasm_bindgen::to_value(&options).unwrap_or_else(|_| JsValue::NULL);

                    match wasm_bindgen_futures::JsFuture::from(startAuthentication(js_options))
                        .await
                    {
                        Ok(credential) => {
                            // Step 3: Send credential to server for verification
                            match auth_service
                                .finish_webauthn_authentication(
                                    session_id,
                                    serde_wasm_bindgen::from_value(credential).unwrap(),
                                )
                                .await
                            {
                                Ok(_) => {
                                    set_loading.set(false);
                                    if let Some(on_success) = on_success {
                                        on_success.call(());
                                    }
                                },
                                Err(e) => {
                                    set_loading.set(false);
                                    set_error
                                        .set(Some(format!("Server verification failed: {}", e)));
                                    if let Some(on_error) = on_error {
                                        on_error.call(e);
                                    }
                                },
                            }
                        },
                        Err(e) => {
                            set_loading.set(false);
                            let error_msg = e
                                .as_string()
                                .unwrap_or_else(|| "Unknown browser error".to_string());
                            set_error.set(Some(error_msg.clone()));
                            if let Some(on_error) = on_error {
                                on_error.call(error_msg);
                            }
                        },
                    }
                },
                Err(e) => {
                    set_loading.set(false);
                    set_error.set(Some(format!("Failed to start authentication: {}", e)));
                    if let Some(on_error) = on_error {
                        on_error.call(e);
                    }
                },
            }
        }
    });

    // Handle cancel button
    let handle_cancel = move |_| {
        if let Some(on_cancel) = on_cancel {
            on_cancel.call(());
        }
    };

    // Component view
    view! { cx,
        <div class="webauthn-authentication-form card p-4">
            <h3 class="card-title text-center">"Use Security Key"</h3>
            <p class="card-subtitle text-center text-muted mb-4">
                "Authenticate with your security key, fingerprint, or facial recognition."
            </p>

            <ErrorDisplay error=error />

            <Show
                when=move || loading.get()
                fallback=move |cx| view! { cx,
                    <div class="d-grid gap-2">
                        <button
                            type="button"
                            class="btn btn-primary"
                            on:click=move |_| authenticate.dispatch(())
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
            >
                <div class="text-center">
                    <LoadingIndicator />
                    <p>"Waiting for your security key..."</p>
                </div>
            </Show>
        </div>
    }
}
