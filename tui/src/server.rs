use std::net::SocketAddr;

use anyhow::Result;
use async_trait::async_trait;
use russh::server::{self, Auth, Msg, Session};
use russh::{Channel, ChannelId, CryptoVec};
use tokio::sync::mpsc;

use crate::events::TuiEvent;
use crate::tui::{self, ChannelMode, SessionStart};

pub struct Client {
    cols: u16,
    rows: u16,
    event_tx: Option<mpsc::UnboundedSender<TuiEvent>>,
    session_started: bool,
}

impl Client {
    fn new() -> Self {
        Self {
            cols: 80,
            rows: 24,
            event_tx: None,
            session_started: false,
        }
    }
}

/// `ssh -t host kana` — remote command string from the client (no `/kana` form).
fn remote_cmd_is_kana(data: &[u8]) -> bool {
    let Ok(s) = std::str::from_utf8(data) else {
        return false;
    };
    let s = s.trim();
    !s.is_empty() && s.eq_ignore_ascii_case("kana")
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

    async fn window_change_request(
        &mut self,
        _channel: ChannelId,
        col_width: u32,
        row_height: u32,
        _pix_width: u32,
        _pix_height: u32,
        _session: &mut Session,
    ) -> Result<(), Self::Error> {
        self.cols = col_width as u16;
        self.rows = row_height as u16;
        if let Some(tx) = &self.event_tx {
            let _ = tx.send(TuiEvent::Resize {
                cols: self.cols,
                rows: self.rows,
            });
        }
        Ok(())
    }

    async fn shell_request(
        &mut self,
        channel: ChannelId,
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        if self.session_started {
            return Ok(());
        }
        self.session_started = true;

        let handle = session.handle();
        let (tx, rx) = mpsc::unbounded_channel();
        self.event_tx = Some(tx);
        let (cols, rows) = (self.cols, self.rows);

        tokio::spawn(async move {
            if let Err(e) = tui::run(
                handle,
                channel,
                rx,
                cols,
                rows,
                SessionStart::Main,
                ChannelMode::Shell,
            )
            .await
            {
                eprintln!("tui error: {e}");
            }
        });

        Ok(())
    }

    async fn exec_request(
        &mut self,
        channel: ChannelId,
        data: &[u8],
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        if !remote_cmd_is_kana(data) {
            let handle = session.handle();
            let msg = b"rulof-tui: unknown remote command (try: ssh -t tui.rulof.dev kana)\n";
            let _ = handle
                .data(channel, CryptoVec::from_slice(msg))
                .await;
            let _ = handle.exit_status_request(channel, 127).await;
            let _ = handle.close(channel).await;
            return Ok(());
        }

        if self.session_started {
            return Ok(());
        }
        self.session_started = true;

        let handle = session.handle();
        let (tx, rx) = mpsc::unbounded_channel();
        self.event_tx = Some(tx);
        let (cols, rows) = (self.cols, self.rows);

        tokio::spawn(async move {
            if let Err(e) = tui::run(
                handle,
                channel,
                rx,
                cols,
                rows,
                SessionStart::Kana,
                ChannelMode::Exec,
            )
            .await
            {
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
        if let Some(tx) = &self.event_tx {
            let _ = tx.send(TuiEvent::Input(data.to_vec()));
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
