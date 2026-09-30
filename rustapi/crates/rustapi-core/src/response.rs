use serde::Serialize;

/// A minimal HTTP response builder with a low-copy byte buffer.
#[derive(Debug, Clone)]
pub struct Response {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl Response {
    pub fn new(status: u16) -> Self {
        Self {
            status,
            headers: Vec::new(),
            body: Vec::new(),
        }
    }

    pub fn ok() -> Self {
        Self::new(200)
    }

    pub fn not_found() -> Self {
        Self::new(404).with_text("Not Found")
    }

    pub fn internal_error() -> Self {
        Self::new(500).with_text("Internal Server Error")
    }

    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push((name.into(), value.into()));
        self
    }

    /// Sets the body without adding any `content-type` header, for callers
    /// that set it explicitly via [`Response::header`].
    pub fn with_body(mut self, bytes: impl Into<Vec<u8>>) -> Self {
        self.body = bytes.into();
        self
    }

    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.body = text.into().into_bytes();
        self.headers
            .push(("content-type".into(), "text/plain; charset=utf-8".into()));
        self
    }

    pub fn with_json<T: Serialize>(mut self, value: &T) -> Result<Self, serde_json::Error> {
        self.body = serde_json::to_vec(value)?;
        self.headers
            .push(("content-type".into(), "application/json".into()));
        Ok(self)
    }

    fn reason_phrase(status: u16) -> &'static str {
        match status {
            200 => "OK",
            201 => "Created",
            204 => "No Content",
            400 => "Bad Request",
            401 => "Unauthorized",
            403 => "Forbidden",
            404 => "Not Found",
            405 => "Method Not Allowed",
            500 => "Internal Server Error",
            _ => "Unknown",
        }
    }

    /// Serializes the response into raw HTTP/1.1 bytes ready to be written to
    /// the socket.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(128 + self.body.len());
        out.extend_from_slice(
            format!(
                "HTTP/1.1 {} {}\r\n",
                self.status,
                Self::reason_phrase(self.status)
            )
            .as_bytes(),
        );
        for (k, v) in &self.headers {
            out.extend_from_slice(format!("{k}: {v}\r\n").as_bytes());
        }
        out.extend_from_slice(format!("content-length: {}\r\n", self.body.len()).as_bytes());
        out.extend_from_slice(b"connection: keep-alive\r\n");
        out.extend_from_slice(b"\r\n");
        out.extend_from_slice(&self.body);
        out
    }
}
