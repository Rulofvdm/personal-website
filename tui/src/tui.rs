use std::io::Write;

use anyhow::Result;
use crossterm::{
    cursor,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
    execute,
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs, Wrap},
    Frame, Terminal, TerminalOptions, Viewport,
};
use russh::{server::Handle, ChannelId, CryptoVec};
use tokio::sync::mpsc;

use crate::content;

// ── Colours (Gruvbox dark) ────────────────────────────────────────────────────

const TEXT: Color       = Color::Rgb(235, 219, 178);
const MUTED: Color      = Color::Rgb(146, 131, 116);
const ACCENT: Color     = Color::Rgb(93, 138, 93);
const ACCENT_STR: Color = Color::Rgb(169, 182, 101);
const SURFACE2: Color   = Color::Rgb(60, 56, 54);

// ── SSH writer ────────────────────────────────────────────────────────────────

struct SshWriter {
    handle: Handle,
    channel: ChannelId,
    buf: Vec<u8>,
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
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let _ = handle.data(channel, data).await;
            });
        });
        Ok(())
    }
}

// ── App state ─────────────────────────────────────────────────────────────────

const TABS: &[&str] = &["about", "experience", "skills", "projects", "contact"];

struct App {
    tab: usize,
}

impl App {
    fn new() -> Self { Self { tab: 0 } }

    fn next(&mut self) { self.tab = (self.tab + 1) % TABS.len(); }

    fn prev(&mut self) {
        if self.tab == 0 { self.tab = TABS.len() - 1; } else { self.tab -= 1; }
    }
}

// ── Input ─────────────────────────────────────────────────────────────────────

fn handle_input(data: &[u8], app: &mut App) -> bool {
    if data == b"\x1b[C" { app.next(); return false; }
    if data == b"\x1b[D" { app.prev(); return false; }
    if data.len() == 1 {
        match data[0] {
            b'q' | b'Q' | 3 => return true, // q, Q, Ctrl-C
            b'l'            => app.next(),
            b'h'            => app.prev(),
            _               => {}
        }
    }
    false
}

// ── Render ────────────────────────────────────────────────────────────────────

fn centered_column(area: ratatui::layout::Rect, max_width: u16) -> ratatui::layout::Rect {
    let width = max_width.min(area.width);
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    ratatui::layout::Rect { x, width, ..area }
}

fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let col = centered_column(area, 90);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(2), // header
            Constraint::Length(2), // tabs
            Constraint::Min(0),    // content
            Constraint::Length(1), // footer
        ])
        .split(col);

    // Header
    frame.render_widget(
        Paragraph::new(vec![
            Line::from(Span::styled("rulof van der merwe", Style::default().fg(TEXT).add_modifier(Modifier::BOLD))),
            Line::from(Span::styled("─────────────────────", Style::default().fg(ACCENT))),
        ]),
        chunks[0],
    );

    // Tabs
    let tab_labels: Vec<Line> = TABS.iter().map(|t| Line::from(Span::raw(*t))).collect();
    frame.render_widget(
        Tabs::new(tab_labels)
            .select(app.tab)
            .style(Style::default().fg(MUTED))
            .highlight_style(Style::default().fg(ACCENT_STR).add_modifier(Modifier::BOLD))
            .divider("  "),
        chunks[1],
    );

    // Content
    let body = match app.tab {
        0 => content::ABOUT,
        1 => content::EXPERIENCE,
        2 => content::SKILLS,
        3 => content::PROJECTS,
        4 => content::CONTACT,
        _ => "",
    };
    frame.render_widget(
        Paragraph::new(body)
            .style(Style::default().fg(TEXT))
            .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(SURFACE2)))
            .wrap(Wrap { trim: true }),
        chunks[2],
    );

    // Footer
    frame.render_widget(
        Paragraph::new(Span::styled(
            "← → or h/l · q to quit",
            Style::default().fg(MUTED),
        )),
        chunks[3],
    );
}

// ── Entry point ───────────────────────────────────────────────────────────────

pub async fn run(
    handle: Handle,
    channel: ChannelId,
    mut input_rx: mpsc::UnboundedReceiver<Vec<u8>>,
    cols: u16,
    rows: u16,
) -> Result<()> {
    let mut writer = SshWriter { handle: handle.clone(), channel, buf: Vec::new() };

    execute!(writer, EnterAlternateScreen, cursor::Hide)?;

    let backend = CrosstermBackend::new(writer);
    let mut terminal = Terminal::with_options(backend, TerminalOptions {
        viewport: Viewport::Fixed(Rect::new(0, 0, cols, rows)),
    })?;

    let mut app = App::new();
    terminal.draw(|f| render(f, &app))?;

    loop {
        match input_rx.recv().await {
            None => break,
            Some(data) => {
                let quit = handle_input(&data, &mut app);
                terminal.draw(|f| render(f, &app))?;
                if quit { break; }
            }
        }
    }

    execute!(terminal.backend_mut(), LeaveAlternateScreen, cursor::Show)?;
    let _ = handle.close(channel).await;

    Ok(())
}
