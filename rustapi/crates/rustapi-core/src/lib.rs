//! Low-level, io_uring-first HTTP core for the `rustapi` framework.
//!
//! This crate intentionally avoids high level HTTP frameworks: requests are
//! parsed directly off the socket with [`httparse`], and the server is built
//! directly on [`monoio`]'s io_uring-backed, thread-per-core runtime.

pub mod error;
pub mod method;
pub mod request;
pub mod response;
pub mod router;
pub mod server;

pub use error::Error;
pub use method::Method;
pub use request::Request;
pub use response::Response;
pub use router::Router;
pub use server::serve;
