use crate::prelude::*;
use crate::view;
use serde::{Deserialize, Serialize};

/// LoginForm structure for data processing
#[derive(Serialize, Deserialize, Clone)]
pub struct LoginForm {
    pub email: String,
    pub password: String,
    pub error: Option<String>,
}

/// Server-side rendered login form component
///
/// This component provides an HTML form for login and
/// is rendered exclusively on the server (SSR).
///
/// # Parameters
///
/// * `cx` - The Leptos scope
/// * `action_path` - The path to which the form is submitted
/// * `error` - An optional error message to be displayed
#[allow(unused_variables)]
pub fn login_form_ssr(cx: Scope, _action_path: String, _error: Option<String>) -> impl IntoView {
    view! { cx,
        <form method="post" action={_action_path} class="auth-form login-form">
            <div class="form-group">
                <label for="email">Email</label>
                <input
                    type="email"
                    id="email"
                    name="email"
                    required
                />
            </div>
            <div class="form-group">
                <label for="password">Password</label>
                <input
                    type="password"
                    id="password"
                    name="password"
                    required
                />
            </div>

            {
                match _error {
                    Some(err) => view! { cx, <div class="error-message">{err}</div> },
                    None => view! { cx, <> </> }
                }
            }

            <div class="form-actions">
                <button type="submit" class="btn btn-primary">Sign In</button>
            </div>

            <div class="form-links">
                <a href="/register" class="register-link">Create Account</a>
            </div>
        </form>
    }
}

// Legacy function to maintain compatibility
#[deprecated(note = "Use login_form_ssr instead")]
pub fn login_form_ssr_legacy(
    cx: Scope,
    action_path: String,
    error: Option<String>,
) -> impl IntoView {
    login_form_ssr(cx, action_path, error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::leptos::ssr::test_utils;

    #[test]
    fn test_login_form_renders_with_correct_structure() {
        // Given a login form with a specific action path
        let action_path = "/api/auth/login".to_string();

        // When rendering the form without an error
        let html = test_utils::render_to_html(|cx| login_form_ssr(cx, action_path.clone(), None));

        // Then it should contain the proper form elements
        assert!(test_utils::assert_has_class(&html, "auth-form"));
        assert!(test_utils::assert_has_class(&html, "login-form"));
        assert!(test_utils::assert_contains_text(&html, "Email"));
        assert!(test_utils::assert_contains_text(&html, "Password"));
        assert!(test_utils::assert_contains_text(&html, "Sign In"));
        assert!(test_utils::assert_contains_text(&html, "/register"));
    }

    #[test]
    fn test_login_form_displays_error_when_provided() {
        // Given a login form with an error message
        let action_path = "/api/auth/login".to_string();
        let error_message = "Invalid credentials".to_string();

        // When rendering the form with the error
        let html = test_utils::render_to_html(|cx| {
            login_form_ssr(cx, action_path.clone(), Some(error_message.clone()))
        });

        // Then it should display the error message
        assert!(test_utils::assert_has_class(&html, "error-message"));
        assert!(test_utils::assert_contains_text(&html, &error_message));
    }

    #[test]
    fn test_login_form_with_custom_action_path() {
        // Given a login form with a custom action path
        let custom_path = "/custom/login/path".to_string();

        // When rendering the form
        let html = test_utils::render_to_html(|cx| login_form_ssr(cx, custom_path.clone(), None));

        // Then it should have the custom action path
        assert!(test_utils::assert_contains_text(&html, &custom_path));
    }
}
