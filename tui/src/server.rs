use std::net::SocketAddr;

use anyhow::Result;
use async_trait::async_trait;
use russh::server::{self, Auth, Msg, Session};
use russh::{Channel, ChannelId};
use tokio::sync::mpsc;

use crate::tui;

pub struct Client {
    cols: u16,
    rows: u16,
    input_tx: Option<mpsc::UnboundedSender<Vec<u8>>>,
}

impl Client {
    fn new() -> Self {
        Self { cols: 80, rows: 24, input_tx: None }
    }
}

#[async_trait]
impl server::Handler for Client {
    type Error = anyhow::Error;

    async fn auth_none(&mut self, _user: &str) -> Result<Auth, Self::Error> {
        Ok(Auth::Accept)
    }

    async fn channel_open_session(
        &mut self,
        _channel: Channel<Msg>,
        _session: &mut Session,
    ) -> Result<bool, Self::Error> {
        Ok(true)
    }

    async fn pty_request(
        &mut self,
        _channel: ChannelId,
        _term: &str,
        col_width: u32,
        row_height: u32,
        _pix_width: u32,
        _pix_height: u32,
        _modes: &[(russh::Pty, u32)],
        _session: &mut Session,
    ) -> Result<(), Self::Error> {
        self.cols = col_width as u16;
        self.rows = row_height as u16;
        Ok(())
    }

    async fn shell_request(
        &mut self,
        channel: ChannelId,
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        let handle = session.handle();
        let (tx, rx) = mpsc::unbounded_channel();
        self.input_tx = Some(tx);
        let (cols, rows) = (self.cols, self.rows);

        tokio::spawn(async move {
            if let Err(e) = tui::run(handle, channel, rx, cols, rows).await {
                eprintln!("tui error: {e}");
            }
        });

        Ok(())
    }

    async fn data(
        &mut self,
        _channel: ChannelId,
        data: &[u8],
        _session: &mut Session,
    ) -> Result<(), Self::Error> {
        if let Some(tx) = &self.input_tx {
            let _ = tx.send(data.to_vec());
        }
        Ok(())
    }
}

pub struct AppServer;

impl server::Server for AppServer {
    type Handler = Client;
    fn new_client(&mut self, _addr: Option<SocketAddr>) -> Self::Handler {
        Client::new()
    }
}
