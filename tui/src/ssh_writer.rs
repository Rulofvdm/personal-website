use std::io::Write;

use russh::{server::Handle, ChannelId, CryptoVec};
use tokio::task::block_in_place;

pub struct SshWriter {
    pub handle: Handle,
    pub channel: ChannelId,
    buf: Vec<u8>,
}

impl SshWriter {
    pub fn new(handle: Handle, channel: ChannelId) -> Self {
        Self {
            handle,
            channel,
            buf: Vec::new(),
        }
    }
}

impl Write for SshWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buf.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        if self.buf.is_empty() {
            return Ok(());
        }
        let data = CryptoVec::from(std::mem::take(&mut self.buf));
        let handle = self.handle.clone();
        let channel = self.channel;
        block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let _ = handle.data(channel, data).await;
            });
        });
        Ok(())
    }
}
