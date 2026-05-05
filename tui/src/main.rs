mod branding;
mod content;
mod events;
mod kana;
mod server;
mod ssh_writer;
mod tui;

use std::path::Path;
use std::sync::Arc;

use anyhow::Result;
use russh::server::Config;
use russh_keys::key::KeyPair;
use tokio::net::TcpListener;

use server::AppServer;

/// Loads `.env` from `tui/` (crate root) or, if missing, from the current working directory.
/// Does not override variables already set in the process environment.
fn load_env_file() {
    let manifest_env = Path::new(env!("CARGO_MANIFEST_DIR")).join(".env");
    if manifest_env.is_file() {
        load_env_from_path(&manifest_env);
        return;
    }
    if let Ok(cwd) = std::env::current_dir() {
        let cwd_env = cwd.join(".env");
        if cwd_env.is_file() {
            load_env_from_path(&cwd_env);
        }
    }
}

fn load_env_from_path(path: &Path) {
    let Ok(contents) = std::fs::read_to_string(path) else {
        return;
    };
    for line in contents.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, raw_val)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        if key.is_empty() || std::env::var_os(key).is_some() {
            continue;
        }
        let mut val = raw_val.trim();
        if val.len() >= 2 {
            let b = val.as_bytes();
            let quoted = (b[0] == b'"' && b[b.len() - 1] == b'"')
                || (b[0] == b'\'' && b[b.len() - 1] == b'\'');
            if quoted {
                val = &val[1..val.len() - 1];
            }
        }
        std::env::set_var(key, val);
    }
}

fn listen_port() -> u16 {
    load_env_file();
    match std::env::var("environment") {
        Ok(v) if v.eq_ignore_ascii_case("development") => 2222,
        _ => 22,
    }
}

fn host_key_path() -> std::path::PathBuf {
    let dir = dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("/tmp"))
        .join(".config/rulof-tui");
    std::fs::create_dir_all(&dir).expect("failed to create config dir");
    dir.join("host_key")
}

fn load_or_create_host_key() -> Result<KeyPair> {
    let path = host_key_path();
    if path.exists() {
        Ok(russh_keys::load_secret_key(&path, None)?)
    } else {
        let key = KeyPair::generate_ed25519().expect("failed to generate host key");
        let file = std::fs::File::create(&path)?;
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

    let port = listen_port();
    let addr = format!("0.0.0.0:{port}");
    let listener = TcpListener::bind(&addr).await?;
    eprintln!("Listening on {addr}");

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
