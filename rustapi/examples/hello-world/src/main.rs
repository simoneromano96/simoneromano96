//! Minimal example showing REST routes (including the `QUERY` method),
//! path/query params, JSON, and OpenAPI/Swagger UI integration.

use rustapi_core::{Request, Response, Router};
use rustapi_openapi::{with_openapi, OpenApi};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
struct User {
    id: u64,
    name: String,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
struct UserQuery {
    name_contains: Option<String>,
}

#[derive(OpenApi)]
#[openapi(components(schemas(User, UserQuery)))]
struct ApiDoc;

fn get_user(req: Request) -> Response {
    let id: u64 = req.param("id").and_then(|s| s.parse().ok()).unwrap_or(0);
    let user = User {
        id,
        name: format!("user-{id}"),
    };
    Response::ok()
        .with_json(&user)
        .unwrap_or_else(|_| Response::internal_error())
}

/// Demonstrates the `QUERY` HTTP method: a safe, cacheable request that
/// carries a JSON body describing a complex filter, instead of encoding it
/// all in the URL as query-string parameters.
fn search_users(req: Request) -> Response {
    let filter: UserQuery = req.json().unwrap_or(UserQuery {
        name_contains: None,
    });

    let all = vec![
        User {
            id: 1,
            name: "alice".into(),
        },
        User {
            id: 2,
            name: "bob".into(),
        },
    ];

    let results: Vec<_> = all
        .into_iter()
        .filter(|u| {
            filter
                .name_contains
                .as_ref()
                .map(|needle| u.name.contains(needle.as_str()))
                .unwrap_or(true)
        })
        .collect();

    Response::ok()
        .with_json(&results)
        .unwrap_or_else(|_| Response::internal_error())
}

fn build_router() -> Router {
    let router = Router::new()
        .get("/users/:id", get_user)
        .query("/users", search_users)
        .get("/health", |_req| Response::ok().with_text("ok"));

    with_openapi(router, "/openapi.json", "/docs", ApiDoc::openapi())
}

fn main() -> std::io::Result<()> {
    println!("listening on http://127.0.0.1:8080 (docs at /docs)");
    rustapi_core::serve("127.0.0.1:8080", build_router)
}
