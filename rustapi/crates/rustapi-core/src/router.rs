use std::collections::HashMap;
use std::rc::Rc;

use crate::method::Method;
use crate::request::Request;
use crate::response::Response;

/// A handler takes a request and produces a response.
///
/// Handlers are `Fn`, not `Fn` + `Send`, because the server runs a
/// thread-per-core, single-threaded io_uring event loop (via `monoio`): there
/// is no cross-thread handoff, so no `Send`/`Sync` bound is required and
/// `Rc`/`RefCell` are fine.
pub type Handler = Rc<dyn Fn(Request) -> Response>;

#[derive(Clone)]
struct Route {
    method: Method,
    segments: Vec<Segment>,
    handler: Handler,
}

#[derive(Clone, PartialEq, Eq)]
enum Segment {
    Static(String),
    Param(String),
}

fn compile_pattern(pattern: &str) -> Vec<Segment> {
    pattern
        .split('/')
        .filter(|s| !s.is_empty())
        .map(|s| {
            if let Some(name) = s.strip_prefix(':') {
                Segment::Param(name.to_string())
            } else {
                Segment::Static(s.to_string())
            }
        })
        .collect()
}

/// A minimal, allocation-conscious method + path router with support for
/// `:param` path segments.
#[derive(Clone, Default)]
pub struct Router {
    routes: Vec<Route>,
}

impl Router {
    pub fn new() -> Self {
        Self { routes: Vec::new() }
    }

    pub fn route(
        mut self,
        method: Method,
        pattern: &str,
        handler: impl Fn(Request) -> Response + 'static,
    ) -> Self {
        self.routes.push(Route {
            method,
            segments: compile_pattern(pattern),
            handler: Rc::new(handler),
        });
        self
    }

    pub fn get(self, pattern: &str, handler: impl Fn(Request) -> Response + 'static) -> Self {
        self.route(Method::Get, pattern, handler)
    }

    pub fn post(self, pattern: &str, handler: impl Fn(Request) -> Response + 'static) -> Self {
        self.route(Method::Post, pattern, handler)
    }

    pub fn put(self, pattern: &str, handler: impl Fn(Request) -> Response + 'static) -> Self {
        self.route(Method::Put, pattern, handler)
    }

    pub fn patch(self, pattern: &str, handler: impl Fn(Request) -> Response + 'static) -> Self {
        self.route(Method::Patch, pattern, handler)
    }

    pub fn delete(self, pattern: &str, handler: impl Fn(Request) -> Response + 'static) -> Self {
        self.route(Method::Delete, pattern, handler)
    }

    /// Registers a handler for the `QUERY` method, a safe, cacheable method
    /// that carries a request body (useful for complex read-only queries).
    pub fn query(self, pattern: &str, handler: impl Fn(Request) -> Response + 'static) -> Self {
        self.route(Method::Query, pattern, handler)
    }

    /// Matches a request against the registered routes and dispatches it.
    pub fn dispatch(&self, mut req: Request) -> Response {
        let path_segments: Vec<&str> = req.path.split('/').filter(|s| !s.is_empty()).collect();

        for route in &self.routes {
            if route.method != req.method {
                continue;
            }
            if route.segments.len() != path_segments.len() {
                continue;
            }

            let mut params = HashMap::new();
            let mut matched = true;
            for (seg, actual) in route.segments.iter().zip(path_segments.iter()) {
                match seg {
                    Segment::Static(s) => {
                        if s != actual {
                            matched = false;
                            break;
                        }
                    }
                    Segment::Param(name) => {
                        params.insert(name.clone(), (*actual).to_string());
                    }
                }
            }

            if matched {
                req.params = params;
                return (route.handler)(req);
            }
        }

        Response::not_found()
    }
}
