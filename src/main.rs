use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

const MAX_REQUEST_SIZE: usize = 8192; // 8 KB safety limit

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:8000").await?;
    println!("🚀 server running on http://localhost:8000");

    loop {
        let (socket, addr) = listener.accept().await?;
        println!("🔗 Connection from {}", addr);

        tokio::spawn(async move {
            if let Err(e) = handle_client(socket).await {
                eprintln!("⚠️  Error handling {}: {}", addr, e);
            }
        });
    }
}

async fn handle_client(mut socket: tokio::net::TcpStream) -> tokio::io::Result<()> {
    let mut buffer = [0u8; MAX_REQUEST_SIZE];

    // Read the request once (simple HTTP/1.1 baseline)
    let n = socket.read(&mut buffer).await?;
    if n == 0 {
        return Ok(()); // closed
    }

    // Parse request
    let req_str = String::from_utf8_lossy(&buffer[..n]);

    // ==== FIRST LINE ====
    let mut lines = req_str.lines();
    let first_line = lines.next().unwrap_or("");
    println!("📥 Request: {}", first_line);

    let mut parts = first_line.split_whitespace();
    let method = parts.next().unwrap_or("");
    let path = parts.next().unwrap_or("/"); // <-- changed from _path

    // ==== BODY EXTRACTION ====
    let full = req_str.as_bytes();
    let header_end = req_str.find("\r\n\r\n").unwrap_or(n);
    let body_start = header_end + 4;

    let body = if body_start < n {
        &full[body_start..n]
    } else {
        &[]
    };

    // Log POST body for demo
    if method == "POST" {
        println!("📦 POST raw body: {}", String::from_utf8_lossy(body));
    }

    // ==== RESPONSE BASED ON PATH & METHOD ====
    let (status, resp_body) = match (method, path) {
        ("GET", "/health") => ("200 OK", r#"{ "status": "healthy" }"#),
        ("GET", "/metrics") => ("200 OK", r#"{ "uptime": 12345, "requests": 987 }"#),
        ("POST", "/order") => {
            // Here you could parse the body JSON for orders
            ("200 OK", r#"{ "status": "order_received" }"#)
        }
        ("GET", "/") => ("200 OK", r#"{ "status": "ok", "method": "GET" }"#),
        ("POST", "/") => ("200 OK", r#"{ "status": "ok", "method": "POST" }"#),
        _ => ("404 NOT FOUND", r#"{ "error": "not_found" }"#),
    };

    // Fixed headers, production-grade
    let response = format!(
        "HTTP/1.1 {status}\r\n\
         Server: ultra-hft/1.0\r\n\
         Content-Type: application/json; charset=utf-8\r\n\
         Content-Length: {}\r\n\
         X-Content-Type-Options: nosniff\r\n\
         X-Frame-Options: DENY\r\n\
         X-XSS-Protection: 1; mode=block\r\n\
         Cache-Control: no-store, no-cache, must-revalidate\r\n\
         Connection: keep-alive\r\n\
         \r\n\
         {resp_body}",
        resp_body.len(),
    );

    socket.write_all(response.as_bytes()).await?;

    Ok(())
}
