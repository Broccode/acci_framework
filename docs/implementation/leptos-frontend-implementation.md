---
title: "Frontend Implementation with Leptos SSR"
author: "Implementation Team"
date: 2025-02-26
status: "draft"
version: "0.1.0"
---

# Frontend Implementation with Leptos (SSR-only)

## Overview

This document outlines the implementation plan for the frontend components of our authentication system using Leptos exclusively in Server-Side Rendering (SSR) mode without WebAssembly. The implementation follows the architectural principles and project goals defined in our documentation.

> ℹ️ **Note:** Our project uses Leptos **exclusively in SSR mode** without WebAssembly compilation. All component rendering occurs on the server-side, and no client-side hydration is performed.

## Current Status

The authentication backend components have been implemented, including:

- User management (registration, login)
- Session handling
- Password management (hashing, validation)
- JWT token handling

The frontend implementation is the next critical step in our development process according to the Milestone 1 plan.

## Implementation Details

### 1. Folder Structure

```
crates/web/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── components/
│   │   ├── mod.rs
│   │   ├── auth/
│   │   │   ├── mod.rs
│   │   │   ├── login_form.rs
│   │   │   └── registration_form.rs
│   │   ├── layout/
│   │   │   ├── mod.rs
│   │   │   ├── navigation.rs
│   │   │   └── footer.rs
│   │   └── common/
│   │       ├── mod.rs
│   │       ├── error_display.rs
│   │       └── loading_indicator.rs
│   ├── pages/
│   │   ├── mod.rs
│   │   ├── home.rs
│   │   ├── login.rs
│   │   └── register.rs
│   ├── utils/
│   │   ├── mod.rs
│   │   ├── validation.rs
│   │   └── form_helpers.rs
│   └── config.rs
└── tests/
    └── component_tests.rs
```

### 2. Component Implementation Plan

#### 2.1 Auth Components

##### Login Form

```rust
use axum::{
    extract::State,
    response::IntoResponse,
    Form,
};
use leptos::*;
use serde::{Deserialize, Serialize};
use acci_auth::LoginCredentials;

#[derive(Serialize, Deserialize, Clone)]
pub struct LoginForm {
    pub email: String,
    pub password: String,
    pub error: Option<String>,
}

/// Server-side rendered login form component
#[component]
pub fn LoginFormSSR(cx: Scope, action_path: String, error: Option<String>) -> impl IntoView {
    view! { cx,
        <form method="post" action={action_path} class="auth-form login-form">
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
            {error.map(|err| view! { cx, <div class="error-message">{err}</div> })}
            <button type="submit" class="btn btn-primary">Login</button>
        </form>
    }
}

// Route handler for login form submission - implemented in axum
pub async fn handle_login(
    State(state): State<AppState>,
    Form(credentials): Form<LoginCredentials>,
) -> impl IntoResponse {
    // Process login using acci_auth services
    // Redirect on success, render form with error on failure
}
```

##### Registration Form

```rust
use axum::{
    extract::State,
    response::IntoResponse,
    Form,
};
use leptos::*;
use serde::{Deserialize, Serialize};
use acci_auth::CreateUser;

#[derive(Serialize, Deserialize, Clone)]
pub struct RegistrationForm {
    pub email: String,
    pub password: String,
    pub password_confirmation: String,
    pub error: Option<String>,
}

/// Server-side rendered registration form component
#[component]
pub fn RegistrationFormSSR(cx: Scope, action_path: String, error: Option<String>) -> impl IntoView {
    view! { cx,
        <form method="post" action={action_path} class="auth-form registration-form">
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
            <div class="form-group">
                <label for="password_confirmation">Confirm Password</label>
                <input 
                    type="password" 
                    id="password_confirmation" 
                    name="password_confirmation" 
                    required
                />
            </div>
            {error.map(|err| view! { cx, <div class="error-message">{err}</div> })}
            <button type="submit" class="btn btn-primary">Register</button>
        </form>
    }
}

// Route handler for registration form submission - implemented in axum
pub async fn handle_registration(
    State(state): State<AppState>,
    Form(form): Form<RegistrationForm>,
) -> impl IntoResponse {
    // Validate form data
    // Process registration using acci_auth services
    // Redirect on success, render form with error on failure
}
```

#### 2.2 Layout Components

##### Navigation

```rust
use leptos::*;
use axum::extract::State;

/// Server-side rendered navigation component
#[component]
pub fn NavigationSSR(cx: Scope, is_authenticated: bool, user_name: Option<String>) -> impl IntoView {
    view! { cx,
        <nav class="main-navigation">
            <div class="logo">
                <a href="/">ACCI Framework</a>
            </div>
            <ul class="nav-links">
                <li><a href="/">Home</a></li>
                {if is_authenticated {
                    view! { cx,
                        <>
                            <li><a href="/dashboard">Dashboard</a></li>
                            <li>
                                <form method="post" action="/logout">
                                    <button type="submit" class="btn-link">Logout</button>
                                </form>
                            </li>
                            <li class="user-info">{user_name.unwrap_or_else(|| "User".to_string())}</li>
                        </>
                    }
                } else {
                    view! { cx,
                        <>
                            <li><a href="/login">Login</a></li>
                            <li><a href="/register">Register</a></li>
                        </>
                    }
                }}
            </ul>
        </nav>
    }
}
```

#### 2.3 Common Components

##### Error Display

```rust
use leptos::*;

#[component]
pub fn ErrorDisplaySSR(cx: Scope, message: String, error_type: Option<String>) -> impl IntoView {
    let type_class = error_type.unwrap_or_else(|| "error".to_string());
    
    view! { cx,
        <div class={format!("error-display {}", type_class)}>
            <div class="error-icon">⚠️</div>
            <div class="error-message">{message}</div>
        </div>
    }
}
```

##### Loading Indicator

```rust
use leptos::*;

#[component]
pub fn LoadingIndicatorSSR(cx: Scope, message: Option<String>) -> impl IntoView {
    let default_message = "Loading...".to_string();
    let display_message = message.unwrap_or(default_message);
    
    view! { cx,
        <div class="loading-indicator">
            <div class="spinner"></div>
            <div class="loading-message">{display_message}</div>
        </div>
    }
}
```

### 3. Page Implementation

#### 3.1 Login Page

```rust
use leptos::*;
use axum::{
    extract::{State, Query},
    response::IntoResponse,
};
use serde::Deserialize;
use crate::components::auth::LoginFormSSR;
use crate::components::layout::NavigationSSR;

#[derive(Deserialize)]
pub struct LoginQuery {
    error: Option<String>,
    redirect: Option<String>,
}

/// Server-side rendered login page
pub async fn login_page(
    State(state): State<AppState>,
    Query(query): Query<LoginQuery>,
) -> impl IntoResponse {
    let renderer = leptos::provide_context(LeptosOptions::default());
    
    let error = query.error;
    let redirect = query.redirect.unwrap_or_else(|| "/".to_string());
    
    // Render the entire page server-side
    leptos::ssr::render_to_string_with_context(
        &renderer,
        move |cx| {
            view! { cx,
                <html>
                    <head>
                        <title>Login - ACCI Framework</title>
                        <link rel="stylesheet" href="/static/styles/main.css" />
                    </head>
                    <body>
                        <NavigationSSR is_authenticated=false user_name=None />
                        <main class="container">
                            <h1>Login</h1>
                            <LoginFormSSR 
                                action_path="/api/auth/login" 
                                error=error 
                            />
                            <div class="form-footer">
                                <p>Don't have an account? <a href="/register">Register</a></p>
                            </div>
                        </main>
                        <script src="/static/js/validation.js"></script>
                    </body>
                </html>
            }
        }
    ).into_response()
}
```

### 4. Integration with Axum

```rust
use axum::{
    routing::get,
    Router,
};
use crate::pages::{home_page, login_page, register_page};
use crate::api::{handle_login, handle_registration, handle_logout};

pub fn create_router() -> Router {
    Router::new()
        // Pages - rendered server-side with Leptos
        .route("/", get(home_page))
        .route("/login", get(login_page))
        .route("/register", get(register_page))
        
        // API endpoints - for form submissions
        .route("/api/auth/login", post(handle_login))
        .route("/api/auth/register", post(handle_registration))
        .route("/api/auth/logout", post(handle_logout))
        
        // Static files
        .nest_service("/static", ServeDir::new("static"))
}
```

### 5. Form Validation

The server will handle all validation, but we'll add minimal client-side validation for better UX:

```javascript
// static/js/validation.js
document.addEventListener('DOMContentLoaded', function() {
  // Simple form validation
  const forms = document.querySelectorAll('form');
  
  forms.forEach(form => {
    form.addEventListener('submit', function(event) {
      const emailInput = form.querySelector('input[type="email"]');
      const passwordInput = form.querySelector('input[type="password"]');
      
      if (emailInput && !isValidEmail(emailInput.value)) {
        event.preventDefault();
        showError(emailInput, 'Please enter a valid email address');
      }
      
      if (passwordInput && passwordInput.value.length < 8) {
        event.preventDefault();
        showError(passwordInput, 'Password must be at least 8 characters');
      }
    });
  });
  
  function isValidEmail(email) {
    const re = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
    return re.test(email);
  }
  
  function showError(input, message) {
    const formGroup = input.closest('.form-group');
    const errorDiv = document.createElement('div');
    errorDiv.className = 'input-error';
    errorDiv.textContent = message;
    
    // Remove any existing error messages
    const existingError = formGroup.querySelector('.input-error');
    if (existingError) {
      formGroup.removeChild(existingError);
    }
    
    formGroup.appendChild(errorDiv);
    input.classList.add('error');
  }
});
```

### 6. Styling

Create basic CSS for the components:

```css
/* static/styles/main.css */
:root {
  --primary-color: #4a6fa5;
  --secondary-color: #166088;
  --accent-color: #4fc08d;
  --text-color: #333;
  --light-text: #777;
  --error-color: #e74c3c;
  --success-color: #2ecc71;
  --warning-color: #f39c12;
  --bg-color: #fff;
  --light-bg: #f5f5f5;
}

/* Container */
.container {
  max-width: 1200px;
  margin: 0 auto;
  padding: 0 20px;
}

/* Navigation */
.main-navigation {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 1rem 2rem;
  background-color: var(--primary-color);
  color: white;
}

.nav-links {
  display: flex;
  list-style: none;
  gap: 1.5rem;
}

.nav-links a {
  color: white;
  text-decoration: none;
}

/* Forms */
.auth-form {
  max-width: 400px;
  margin: 2rem auto;
  padding: 2rem;
  border-radius: 8px;
  box-shadow: 0 2px 10px rgba(0, 0, 0, 0.1);
  background-color: var(--bg-color);
}

.form-group {
  margin-bottom: 1.5rem;
}

.form-group label {
  display: block;
  margin-bottom: 0.5rem;
  font-weight: 500;
}

.form-group input {
  width: 100%;
  padding: 0.75rem;
  border: 1px solid #ddd;
  border-radius: 4px;
  font-size: 1rem;
}

.btn {
  padding: 0.75rem 1.5rem;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 1rem;
  font-weight: 500;
}

.btn-primary {
  background-color: var(--primary-color);
  color: white;
}

.btn-link {
  background: none;
  border: none;
  color: var(--primary-color);
  text-decoration: underline;
  cursor: pointer;
  padding: 0;
  font: inherit;
}

/* Error display */
.error-display {
  display: flex;
  align-items: center;
  padding: 1rem;
  margin-bottom: 1.5rem;
  border-radius: 4px;
  background-color: rgba(231, 76, 60, 0.1);
  color: var(--error-color);
}

.error-icon {
  margin-right: 0.75rem;
  font-size: 1.25rem;
}

.error-message {
  color: var(--error-color);
  margin-top: 0.5rem;
  font-size: 0.9rem;
}

.input-error {
  color: var(--error-color);
  font-size: 0.85rem;
  margin-top: 0.5rem;
}

input.error {
  border-color: var(--error-color);
}

/* Loading indicator */
.loading-indicator {
  display: flex;
  flex-direction: column;
  align-items: center;
  margin: 2rem 0;
}

.spinner {
  width: 40px;
  height: 40px;
  border: 4px solid rgba(0, 0, 0, 0.1);
  border-radius: 50%;
  border-top-color: var(--primary-color);
  animation: spin 1s ease-in-out infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.loading-message {
  margin-top: 1rem;
  color: var(--light-text);
}
```

## Implementation Schedule

### Week 1: Basic Components and Structure

- [ ] Set up the basic folder structure
- [ ] Create the core components (login form, registration form)
- [ ] Implement basic CSS styling
- [ ] Integrate with Axum for routing

### Week 2: Full Page Implementation and Testing

- [ ] Implement all pages (home, login, register)
- [ ] Add form validation
- [ ] Add error handling components
- [ ] Create unit tests for components

### Week 3: Integration with Auth Service

- [ ] Connect forms to authentication service
- [ ] Implement session handling in UI
- [ ] Add conditional rendering based on auth state
- [ ] Implement logout functionality

### Week 4: Polish and Testing

- [ ] Add responsive design
- [ ] Implement accessibility features
- [ ] Create end-to-end tests
- [ ] Performance optimization

## Important SSR Considerations

Since we are exclusively using Leptos in SSR mode without WebAssembly, the following considerations apply:

1. **All rendering occurs on the server** - There is no client-side hydration
2. **Form submissions use standard HTML forms** - Not Leptos events
3. **Client-side interactivity is limited** - Use minimal JavaScript for form validation only
4. **Session state is managed through cookies** - Not client-side state
5. **Page navigation causes full page reloads** - Not SPA navigation

## Testing Strategy

### Unit Tests

```rust
use leptos::*;
use crate::components::auth::LoginFormSSR;
use wasm_bindgen_test::*;

#[test]
fn test_login_form_renders_correctly() {
    // SSR rendering test
    let renderer = leptos::provide_context(LeptosOptions::default());
    
    let html = leptos::ssr::render_to_string_with_context(
        &renderer,
        |cx| {
            view! { cx,
                <LoginFormSSR action_path="/login" error=None />
            }
        }
    );
    
    // Assert specific elements exist in the rendered HTML
    assert!(html.contains("<form"));
    assert!(html.contains("method=\"post\""));
    assert!(html.contains("action=\"/login\""));
    assert!(html.contains("type=\"email\""));
    assert!(html.contains("type=\"password\""));
}

#[test]
fn test_login_form_displays_error() {
    // SSR rendering test with error
    let renderer = leptos::provide_context(LeptosOptions::default());
    
    let error = Some("Invalid credentials".to_string());
    
    let html = leptos::ssr::render_to_string_with_context(
        &renderer,
        move |cx| {
            view! { cx,
                <LoginFormSSR action_path="/login" error=error />
            }
        }
    );
    
    // Assert error message is displayed
    assert!(html.contains("Invalid credentials"));
    assert!(html.contains("error-message"));
}
```

### Integration Tests

```rust
use crate::create_app;
use axum::extract::Request;
use axum::body::Body;
use http::Request;
use tower::ServiceExt;

#[tokio::test]
async fn test_login_page_renders() {
    let app = create_app().await;
    
    let response = app
        .oneshot(Request::builder().uri("/login").body(Body::empty()).unwrap())
        .await
        .unwrap();
    
    assert_eq!(response.status(), 200);
    
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    
    // Check for expected content
    assert!(body_str.contains("<title>Login"));
    assert!(body_str.contains("<form method=\"post\""));
}

#[tokio::test]
async fn test_login_with_invalid_credentials() {
    let app = create_app().await;
    
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/auth/login")
                .method("POST")
                .header("Content-Type", "application/x-www-form-urlencoded")
                .body(Body::from("email=test%40example.com&password=wrongpassword"))
                .unwrap()
        )
        .await
        .unwrap();
    
    // Should redirect back to login page with error
    assert_eq!(response.status(), 302);
    assert_eq!(
        response.headers().get("Location").unwrap(),
        "/login?error=Invalid+credentials"
    );
}
```

## Conclusion

This implementation plan outlines how to build the frontend components for our authentication system using Leptos exclusively in SSR mode. By following this plan, we'll create a secure, performant, and maintainable UI that integrates seamlessly with our backend authentication services.

Remember that all rendering occurs on the server, which simplifies our architecture by avoiding the complexities of client-side state management and hydration that would be present in a WebAssembly-based approach.
