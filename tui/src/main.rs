mod content;
mod server;
mod tui;

use std::sync::Arc;

use anyhow::Result;
use russh::server::Config;
use russh_keys::key::KeyPair;
use tokio::net::TcpListener;

use server::AppServer;

const HOST_KEY_PATH: &str = "host_key";

fn load_or_create_host_key() -> Result<KeyPair> {
    if std::path::Path::new(HOST_KEY_PATH).exists() {
        Ok(russh_keys::load_secret_key(HOST_KEY_PATH, None)?)
    } else {
        let key = KeyPair::generate_ed25519().expect("failed to generate host key");
        let file = std::fs::File::create(HOST_KEY_PATH)?;
        russh_keys::encode_pkcs8_pem(&key, file)?;
        Ok(key)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let key = load_or_create_host_key()?;

    let config = Arc::new(Config {
        keys: vec![key],
        ..Default::default()
    });

    let listener = TcpListener::bind("0.0.0.0:2222").await?;
    eprintln!("Listening on 0.0.0.0:2222");

    let mut srv = AppServer;

    loop {
        let (stream, addr) = listener.accept().await?;
        let config = config.clone();
        let handler = russh::server::Server::new_client(&mut srv, Some(addr));
        tokio::spawn(async move {
            if let Err(e) = russh::server::run_stream(config, stream, handler).await {
                eprintln!("connection error: {e}");
            }
        });
    }
}
