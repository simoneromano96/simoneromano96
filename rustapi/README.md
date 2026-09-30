# rustapi

A low-level, `io_uring`-first HTTP framework for Rust, with first-class REST,
GraphQL, and OpenAPI support.

> ⚠️ Early scaffold. The core server, router, and OpenAPI integration are
> functional (see `examples/hello-world`), but this is not production-ready
> yet. GraphQL integration and benchmarks are still to come.

## Why

Most Rust web frameworks (`axum`, `actix-web`, ...) are built on `tokio` and
`hyper`. `rustapi` instead:

* Parses HTTP/1.1 directly off the socket with [`httparse`](https://docs.rs/httparse)
  — no intermediate high-level HTTP crate.
* Runs on [`monoio`](https://github.com/bytedance/monoio), an `io_uring`-first,
  thread-per-core async runtime (falling back to epoll/kqueue via monoio's
  legacy driver on kernels/platforms without `io_uring` support).
* Uses `Rc`-based, non-`Send` handlers, since each worker thread owns its own
  event loop and there is no cross-thread task handoff — no `Arc`/`Mutex`
  overhead on the hot path.

## Workspace layout

| Crate                              | Purpose                                                        |
| ----------------------------------- | ---------------------------------------------------------------- |
| `crates/rustapi-core`               | HTTP parsing, router, and the `io_uring` server loop             |
| `crates/rustapi-openapi`            | OpenAPI JSON + Swagger UI route helpers, built on [`utoipa`](https://docs.rs/utoipa) |
| `examples/hello-world`              | End-to-end example: REST routes, `QUERY` method, OpenAPI docs   |

GraphQL support (via [`async-graphql`](https://docs.rs/async-graphql)) is
planned as a `rustapi-graphql` crate.

## Quick start

```bash
cargo run -p hello-world
```

```bash
curl http://127.0.0.1:8080/health
curl http://127.0.0.1:8080/users/42

# The QUERY method: a safe, cacheable request that carries a JSON body,
# useful for complex read-only filters that don't fit in a URL.
curl -X QUERY http://127.0.0.1:8080/users \
  -H 'content-type: application/json' \
  -d '{"name_contains":"ali"}'

# OpenAPI document + Swagger UI
curl http://127.0.0.1:8080/openapi.json
open http://127.0.0.1:8080/docs
```

## Defining routes

```rust
use rustapi_core::{Request, Response, Router};

fn get_user(req: Request) -> Response {
    let id = req.param("id").unwrap_or("0");
    Response::ok().with_text(format!("user {id}"))
}

fn build_router() -> Router {
    Router::new()
        .get("/users/:id", get_user)
        .query("/users", search_users) // QUERY method
}

fn main() -> std::io::Result<()> {
    rustapi_core::serve("127.0.0.1:8080", build_router)
}
```

## OpenAPI integration

```rust
use rustapi_openapi::{with_openapi, OpenApi};

#[derive(OpenApi)]
#[openapi(components(schemas(User)))]
struct ApiDoc;

let router = with_openapi(router, "/openapi.json", "/docs", ApiDoc::openapi());
```

## Supported HTTP methods

`GET`, `POST`, `PUT`, `PATCH`, `DELETE`, and the IETF draft
[`QUERY`](https://datatracker.ietf.org/doc/draft-ietf-httpbis-safe-method-w-body/)
method — a safe, idempotent, cacheable method for expressing complex
read-only queries with a request body.

## Roadmap

* [ ] GraphQL execution (`async-graphql`) wired into the router
* [ ] HTTP/2 support
* [ ] Criterion micro-benchmarks + `wrk`/`oha` throughput comparisons against
      `axum`/`actix-web`
* [ ] CI (build, test, clippy, fmt)

## License

MIT
