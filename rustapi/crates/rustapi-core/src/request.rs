use std::collections::HashMap;

use crate::method::Method;

/// A parsed, immutable HTTP request.
///
/// Parsing is performed with `httparse` directly on the bytes read from the
/// socket, avoiding any intermediate high level HTTP abstraction.
#[derive(Debug, Clone)]
pub struct Request {
    pub method: Method,
    pub path: String,
    pub query: HashMap<String, String>,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
    /// Path parameters extracted by the router (e.g. `/users/:id`).
    pub params: HashMap<String, String>,
}

impl Request {
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
    }

    pub fn param(&self, name: &str) -> Option<&str> {
        self.params.get(name).map(|s| s.as_str())
    }

    pub fn query_param(&self, name: &str) -> Option<&str> {
        self.query.get(name).map(|s| s.as_str())
    }

    pub fn json<T: serde::de::DeserializeOwned>(&self) -> Result<T, serde_json::Error> {
        serde_json::from_slice(&self.body)
    }
}

/// Splits `path?a=1&b=2` into (`path`, query map).
pub(crate) fn split_path_and_query(raw: &str) -> (String, HashMap<String, String>) {
    let mut parts = raw.splitn(2, '?');
    let path = parts.next().unwrap_or("/").to_string();
    let mut query = HashMap::new();
    if let Some(q) = parts.next() {
        for pair in q.split('&') {
            if pair.is_empty() {
                continue;
            }
            let mut kv = pair.splitn(2, '=');
            let k = kv.next().unwrap_or_default();
            let v = kv.next().unwrap_or_default();
            query.insert(urlencoding_decode(k), urlencoding_decode(v));
        }
    }
    (path, query)
}

/// Minimal percent-decoding (no external dependency, low level by design).
fn urlencoding_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            b'%' if i + 2 < bytes.len() => {
                if let Ok(hex) =
                    u8::from_str_radix(std::str::from_utf8(&bytes[i + 1..i + 3]).unwrap_or(""), 16)
                {
                    out.push(hex);
                    i += 3;
                } else {
                    out.push(bytes[i]);
                    i += 1;
                }
            }
            b => {
                out.push(b);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Parses a raw HTTP/1.1 request buffer using `httparse`.
///
/// Returns the parsed [`Request`] and the number of header bytes consumed, so
/// the caller can slice out the body (which may still be incomplete for
/// chunked/streamed bodies — left for a future iteration).
pub fn parse_request(buf: &[u8]) -> Result<Option<(Request, usize)>, crate::Error> {
    let mut headers = [httparse::EMPTY_HEADER; 64];
    let mut req = httparse::Request::new(&mut headers);

    let status = req
        .parse(buf)
        .map_err(|e| crate::Error::Parse(e.to_string()))?;

    let consumed = match status {
        httparse::Status::Complete(n) => n,
        httparse::Status::Partial => return Ok(None),
    };

    let method = Method::parse(req.method.unwrap_or("GET"));
    let (path, query) = split_path_and_query(req.path.unwrap_or("/"));

    let mut content_length = 0usize;
    let mut out_headers = Vec::with_capacity(req.headers.len());
    for h in req.headers.iter() {
        let name = h.name.to_string();
        let value = String::from_utf8_lossy(h.value).into_owned();
        if name.eq_ignore_ascii_case("content-length") {
            content_length = value.trim().parse().unwrap_or(0);
        }
        out_headers.push((name, value));
    }

    let body_start = consumed;
    let available_body = buf.len().saturating_sub(body_start);
    if available_body < content_length {
        // Body not fully received yet.
        return Ok(None);
    }

    let body = buf[body_start..body_start + content_length].to_vec();

    Ok(Some((
        Request {
            method,
            path,
            query,
            headers: out_headers,
            body,
            params: HashMap::new(),
        },
        body_start + content_length,
    )))
}
