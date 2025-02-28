use crate::config::ApiConfig;
use axum::{Router, response::Html, routing::get};

/// API documentation server
pub struct ApiDocumentation {
    config: ApiConfig,
}

impl ApiDocumentation {
    /// Creates a new API documentation server
    pub fn new(config: ApiConfig) -> Self {
        Self { config }
    }

    /// Registers the documentation routes
    pub fn register_routes(&self, router: Router) -> Router {
        if self.config.documentation.enabled {
            router.nest(&self.config.documentation.path, self.documentation_routes())
        } else {
            router
        }
    }

    /// Creates the documentation routes
    fn documentation_routes(&self) -> Router {
        Router::new()
            .route("/", get(swagger_ui_handler))
            .route("/openapi.json", get(openapi_json_handler))
    }
}

/// Handler for the Swagger UI
async fn swagger_ui_handler() -> Html<String> {
    Html(r#"
        <!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="UTF-8">
            <title>ACCI Framework - API Documentation</title>
            <link rel="stylesheet" type="text/css" href="https://cdn.jsdelivr.net/npm/swagger-ui-dist@5.4.2/swagger-ui.css">
            <style>
                html { box-sizing: border-box; overflow: -moz-scrollbars-vertical; overflow-y: scroll; }
                *, *:before, *:after { box-sizing: inherit; }
                body { margin: 0; background: #fafafa; }
                .swagger-ui .topbar { display: none; }
            </style>
        </head>
        <body>
            <div id="swagger-ui"></div>
            <script src="https://cdn.jsdelivr.net/npm/swagger-ui-dist@5.4.2/swagger-ui-bundle.js"></script>
            <script src="https://cdn.jsdelivr.net/npm/swagger-ui-dist@5.4.2/swagger-ui-standalone-preset.js"></script>
            <script>
                window.onload = function() {
                    window.ui = SwaggerUIBundle({
                        url: "./openapi.json",
                        dom_id: '#swagger-ui',
                        deepLinking: true,
                        presets: [
                            SwaggerUIBundle.presets.apis,
                            SwaggerUIStandalonePreset
                        ],
                        plugins: [
                            SwaggerUIBundle.plugins.DownloadUrl
                        ],
                        layout: "BaseLayout",
                        filter: true,
                        withCredentials: true,
                    });
                };
            </script>
        </body>
        </html>
    "#.to_string())
}

/// Handler for the OpenAPI JSON specification
async fn openapi_json_handler() -> axum::Json<serde_json::Value> {
    // Simple OpenAPI specification
    axum::Json(serde_json::json!({
        "openapi": "3.0.3",
        "info": {
            "title": "ACCI Framework API",
            "description": "API documentation for the ACCI Framework",
            "version": "1.0.0"
        },
        "paths": {
            "/api/v1/health": {
                "get": {
                    "summary": "Health check",
                    "description": "Verifies that the API is functioning",
                    "operationId": "healthCheck",
                    "responses": {
                        "200": {
                            "description": "Successful response",
                            "content": {
                                "text/plain": {
                                    "schema": {
                                        "type": "string",
                                        "example": "OK"
                                    }
                                }
                            }
                        }
                    }
                }
            }
        },
        "components": {
            "schemas": {
                "ApiResponse": {
                    "type": "object",
                    "properties": {
                        "status": {
                            "type": "string",
                            "enum": ["success", "error"],
                            "description": "Response status"
                        },
                        "data": {
                            "type": "object",
                            "description": "Response data (only for successful responses)"
                        },
                        "message": {
                            "type": "string",
                            "description": "Error message (only for error responses)"
                        },
                        "code": {
                            "type": "string",
                            "description": "Error code (only for error responses)"
                        },
                        "request_id": {
                            "type": "string",
                            "description": "Unique request ID"
                        }
                    },
                    "required": ["status", "request_id"]
                }
            }
        }
    }))
}
