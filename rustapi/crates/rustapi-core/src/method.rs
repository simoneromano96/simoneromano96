use std::fmt;

/// HTTP request method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Method {
    Get,
    Post,
    Put,
    Patch,
    Delete,
    Head,
    Options,
    /// The `QUERY` method (draft-ietf-httpbis-safe-method-w-body): a safe,
    /// idempotent, cacheable method that carries a request body, typically
    /// used to express complex read-only queries that don't fit in a URL.
    Query,
    Other,
}

impl Method {
    pub fn parse(raw: &str) -> Self {
        match raw {
            "GET" => Method::Get,
            "POST" => Method::Post,
            "PUT" => Method::Put,
            "PATCH" => Method::Patch,
            "DELETE" => Method::Delete,
            "HEAD" => Method::Head,
            "OPTIONS" => Method::Options,
            "QUERY" => Method::Query,
            _ => Method::Other,
        }
    }
}

impl fmt::Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Method::Get => "GET",
            Method::Post => "POST",
            Method::Put => "PUT",
            Method::Patch => "PATCH",
            Method::Delete => "DELETE",
            Method::Head => "HEAD",
            Method::Options => "OPTIONS",
            Method::Query => "QUERY",
            Method::Other => "OTHER",
        };
        write!(f, "{s}")
    }
}
