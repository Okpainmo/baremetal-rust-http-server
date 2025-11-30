use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    // Bind to 0.0.0.0:8000
    let listener = TcpListener::bind("0.0.0.0:8000").await?;
    println!("🚀 Server running on http://localhost:8000");

    loop {
        // Wait for an incoming connection
        let (mut socket, addr) = listener.accept().await?;
        println!("🔗 Connection from {}", addr);

        // Spawn a task so multiple clients can be handled concurrently
        tokio::spawn(async move {
            let mut buffer = [0u8; 1024];

            // Read the request bytes
            let n = match socket.read(&mut buffer).await {
                Ok(n) if n == 0 => return, // client closed
                Ok(n) => n,
                Err(_) => return, // ignore faulty connection
            };

            // Convert to string for simple parsing
            let request = String::from_utf8_lossy(&buffer[..n]);
            let first_line = request.lines().next().unwrap_or("");
            println!("📥 Request: {}", first_line);

            // Prepare JSON response
            let body = r#"{ "status": "ok" }"#;
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                body.len(),
                body
            );

            // Send the response
            if socket.write_all(response.as_bytes()).await.is_err() {
                println!("⚠️  Failed to send response to {}", addr);
            }
        });
    }
}
