use clap::Parser;
use std::net::{IpAddr, SocketAddr};
// Use the logic from lib.rs
// Assuming your library crate is named "bloomsrv" in Cargo.toml
use bloomsrv::{create_app, SharedState};

const DEFAULT_HOST: &str = "127.0.0.1";
const DEFAULT_PORT: u16 = 3000;

/// Simple Bloom Filter Daemon
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Host to listen on
    #[arg(long, env = "BLOOMSRV_HOST", default_value = DEFAULT_HOST)]
    host: IpAddr,

    /// Port to listen on
    #[arg(short, long, env = "BLOOMSRV_PORT", default_value_t = DEFAULT_PORT)]
    port: u16,
}

#[tokio::main]
async fn main() {
    // Parse command line arguments (and environment variables)
    let args = Args::parse();

    let state = SharedState::default();

    // We use the public function from lib.rs
    let app = create_app(state);

    let addr = SocketAddr::from((args.host, args.port));
    println!("Bloom Daemon listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
