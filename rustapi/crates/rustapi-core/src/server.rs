use std::rc::Rc;

use monoio::io::{AsyncReadRent, AsyncWriteRentExt};
use monoio::net::{TcpListener, TcpStream};

use crate::request::parse_request;
use crate::response::Response;
use crate::router::Router;

/// Runs the HTTP server on the given address, using one `monoio` (io_uring)
/// event loop per CPU core (thread-per-core), which avoids cross-thread
/// synchronization on the hot request path entirely.
///
/// `router_factory` is invoked once per worker thread to build a thread-local
/// [`Router`], since handlers are `Rc`-based and therefore not `Send`.
pub fn serve<F>(addr: &str, router_factory: F) -> std::io::Result<()>
where
    F: Fn() -> Router + Send + Clone + 'static,
{
    let cores = num_cpus();
    let addr = addr.to_string();

    let mut threads = Vec::with_capacity(cores);
    for _ in 0..cores {
        let addr = addr.clone();
        let router_factory = router_factory.clone();
        threads.push(std::thread::spawn(move || {
            let router = Rc::new(router_factory());
            run_on_current_thread(&addr, router)
        }));
    }

    for t in threads {
        t.join().expect("worker thread panicked")?;
    }
    Ok(())
}

fn num_cpus() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
}

fn run_on_current_thread(addr: &str, router: Rc<Router>) -> std::io::Result<()> {
    match monoio::RuntimeBuilder::<monoio::IoUringDriver>::new()
        .enable_timer()
        .build()
    {
        Ok(mut rt) => {
            let addr = addr.to_string();
            rt.block_on(async move { accept_loop(&addr, router).await })
        }
        Err(_) => {
            // Fall back to the legacy (epoll/kqueue) driver on platforms or
            // kernels without io_uring support.
            let mut rt = monoio::RuntimeBuilder::<monoio::LegacyDriver>::new()
                .enable_timer()
                .build()?;
            let addr = addr.to_string();
            rt.block_on(async move { accept_loop(&addr, router).await })
        }
    }
}

async fn accept_loop(addr: &str, router: Rc<Router>) -> std::io::Result<()> {
    let listener = TcpListener::bind(addr)?;
    loop {
        let (stream, _peer) = listener.accept().await?;
        let router = router.clone();
        monoio::spawn(async move {
            if let Err(_e) = handle_connection(stream, router).await {
                // Connection closed or errored; nothing else to do.
            }
        });
    }
}

async fn handle_connection(mut stream: TcpStream, router: Rc<Router>) -> std::io::Result<()> {
    let mut buf: Vec<u8> = Vec::with_capacity(8 * 1024);
    loop {
        let read_buf = std::mem::take(&mut buf);
        let (res, mut read_buf) = read_into(&mut stream, read_buf).await;
        let n = res?;
        if n == 0 {
            return Ok(());
        }

        loop {
            match parse_request(&read_buf) {
                Ok(Some((request, consumed))) => {
                    let response = router.dispatch(request);
                    let bytes = response.to_bytes();
                    let (res, _) = stream.write_all(bytes).await;
                    res?;

                    read_buf.drain(0..consumed);
                    if read_buf.is_empty() {
                        break;
                    }
                    // Try to parse another pipelined request from what's left.
                    continue;
                }
                Ok(None) => {
                    // Incomplete request; read more bytes.
                    break;
                }
                Err(_) => {
                    let response = Response::new(400).with_text("Bad Request");
                    let (res, _) = stream.write_all(response.to_bytes()).await;
                    res?;
                    return Ok(());
                }
            }
        }

        buf = read_buf;
        buf.reserve(8 * 1024);
    }
}

/// Reads available bytes from the socket, appending them to `acc`.
async fn read_into(stream: &mut TcpStream, mut acc: Vec<u8>) -> (std::io::Result<usize>, Vec<u8>) {
    let start = acc.len();
    acc.resize(start + 8 * 1024, 0);
    let (res, mut chunk) = stream.read(acc.split_off(start)).await;
    match res {
        Ok(n) => {
            chunk.truncate(n);
            acc.extend_from_slice(&chunk);
            (Ok(n), acc)
        }
        Err(e) => (Err(e), acc),
    }
}
