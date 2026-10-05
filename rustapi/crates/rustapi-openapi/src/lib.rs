//! OpenAPI integration for `rustapi`.
//!
//! This crate stays intentionally thin: it re-exports [`utoipa`]'s derive
//! macros for schema/path generation and provides two small `rustapi_core`
//! route handlers to serve the resulting spec as JSON and as an interactive
//! Swagger UI page, without pulling in a framework-specific UI integration.

use rustapi_core::{Response, Router};
use utoipa::openapi::OpenApi as OpenApiSpec;

pub use utoipa;
pub use utoipa::OpenApi;

/// Registers `GET {json_path}` (serving the OpenAPI document as JSON) and
/// `GET {ui_path}` (serving an interactive Swagger UI page backed by the
/// `swagger-ui` CDN bundle, pointed at `json_path`) on the given router.
pub fn with_openapi(
    router: Router,
    json_path: &'static str,
    ui_path: &'static str,
    spec: OpenApiSpec,
) -> Router {
    let json = spec.to_json().unwrap_or_default();
    let ui_html = swagger_ui_html(json_path);

    router
        .get(json_path, move |_req| {
            Response::ok()
                .header("content-type", "application/json")
                .with_body(json.clone().into_bytes())
        })
        .get(ui_path, move |_req| {
            Response::ok()
                .header("content-type", "text/html; charset=utf-8")
                .with_body(ui_html.clone().into_bytes())
        })
}

fn swagger_ui_html(spec_url: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html>
  <head>
    <title>rustapi - Swagger UI</title>
    <link rel="stylesheet" href="https://unpkg.com/swagger-ui-dist/swagger-ui.css" />
  </head>
  <body>
    <div id="swagger-ui"></div>
    <script src="https://unpkg.com/swagger-ui-dist/swagger-ui-bundle.js"></script>
    <script>
      window.onload = () => {{
        window.ui = SwaggerUIBundle({{
          url: '{spec_url}',
          dom_id: '#swagger-ui',
        }});
      }};
    </script>
  </body>
</html>"#
    )
}
