use anyhow::Result;
use crossterm::{
    cursor,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
    execute,
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs, Wrap},
    Frame, Terminal, TerminalOptions, Viewport,
};
use russh::{server::Handle, ChannelId};
use tokio::sync::mpsc;

use crate::branding::{self, render_footer_hint, render_site_header, ACCENT_STR, MUTED, SURFACE2, TEXT};
use crate::content;
use crate::events::TuiEvent;
use crate::kana;
use crate::ssh_writer::SshWriter;

const TABS: &[&str] = &["about", "experience", "skills", "projects", "contact"];

const PALETTE: &[(&str, GoTo, bool)] = &[
    ("about", GoTo::Section(0), false),
    ("experience", GoTo::Section(1), false),
    ("skills", GoTo::Section(2), false),
    ("projects", GoTo::Section(3), false),
    ("contact", GoTo::Section(4), false),
    ("kana", GoTo::Kana, true),
];

#[derive(Clone, Copy)]
pub(crate) enum GoTo {
    Section(usize),
    Kana,
}

#[derive(Clone, Copy, Default)]
pub enum SessionStart {
    #[default]
    Main,
    /// Open the kana drill first; return to main when it ends.
    Kana,
}

/// Whether this session was opened as an interactive shell or a remote command (`exec`).
#[derive(Clone, Copy, Default)]
pub enum ChannelMode {
    #[default]
    Shell,
    Exec,
}

struct PaletteState {
    query: String,
    /// Index into the filtered list from [`palette_filtered`].
    selected: usize,
}

impl PaletteState {
    fn new() -> Self {
        Self {
            query: String::new(),
            selected: 0,
        }
    }
}

struct App {
    tab: usize,
    palette: Option<PaletteState>,
}

impl App {
    fn new() -> Self {
        Self {
            tab: 0,
            palette: None,
        }
    }

    fn next(&mut self) {
        self.tab = (self.tab + 1) % TABS.len();
    }

    fn prev(&mut self) {
        if self.tab == 0 {
            self.tab = TABS.len() - 1;
        } else {
            self.tab -= 1;
        }
    }
}

fn palette_filtered(query: &str) -> Vec<(usize, GoTo, bool)> {
    let q = query.to_ascii_lowercase();
    PALETTE
        .iter()
        .enumerate()
        .filter(|(_, (cmd, _, _))| q.is_empty() || cmd.starts_with(&q))
        .map(|(i, &(_, goto, secret))| (i, goto, secret))
        .collect()
}

enum SiteAction {
    Quit,
    Handled,
    PassThrough,
}

fn handle_site_input(data: &[u8], app: &mut App) -> SiteAction {
    if data == b"\x1b[C" {
        app.next();
        return SiteAction::Handled;
    }
    if data == b"\x1b[D" {
        app.prev();
        return SiteAction::Handled;
    }
    if data.len() == 1 {
        match data[0] {
            b'q' | b'Q' | 3 => return SiteAction::Quit,
            b'l' => {
                app.next();
                return SiteAction::Handled;
            }
            b'h' => {
                app.prev();
                return SiteAction::Handled;
            }
            _ => {}
        }
    }
    SiteAction::PassThrough
}

enum PaletteAction {
    /// Close the palette and stay on the current screen.
    Dismiss,
    /// Close the palette and perform navigation.
    Go(GoTo),
    /// Keep the palette open (redraw).
    Stay,
}

fn handle_palette_input(data: &[u8], p: &mut PaletteState) -> PaletteAction {
    let filtered = palette_filtered(&p.query);

    if data == b"\x1b[A" {
        if !filtered.is_empty() {
            p.selected = p.selected.saturating_sub(1);
        }
        return PaletteAction::Stay;
    }
    if data == b"\x1b[B" {
        if !filtered.is_empty() {
            let max = filtered.len() - 1;
            p.selected = (p.selected + 1).min(max);
        }
        return PaletteAction::Stay;
    }

    if data.len() == 1 {
        match data[0] {
            0x1b | 3 => return PaletteAction::Dismiss,
            b'q' | b'Q' => return PaletteAction::Dismiss,
            b'\r' | b'\n' => {
                if let Some(&(_, goto, _)) = filtered.get(p.selected) {
                    return PaletteAction::Go(goto);
                }
                return PaletteAction::Stay;
            }
            0x7f | 0x08 => {
                p.query.pop();
                p.selected = 0;
                return PaletteAction::Stay;
            }
            b if b.is_ascii_graphic() => {
                p.query.push(b as char);
                p.selected = 0;
                let next = palette_filtered(&p.query);
                if !next.is_empty() && p.selected >= next.len() {
                    p.selected = next.len() - 1;
                }
                return PaletteAction::Stay;
            }
            _ => return PaletteAction::Stay,
        }
    }

    PaletteAction::Stay
}

/// Typing `kana` (any mix of case) launches the hidden drill. Skips chunks that look like escapes.
struct KanaEgg {
    buf: String,
}

impl KanaEgg {
    fn new() -> Self {
        Self { buf: String::new() }
    }

    fn feed(&mut self, data: &[u8]) -> bool {
        if data.contains(&0x1b) {
            return false;
        }
        for &b in data {
            if !b.is_ascii_alphabetic() {
                continue;
            }
            let c = (b as char).to_ascii_lowercase();
            self.buf.push(c);
            while self.buf.len() > 4 {
                self.buf.remove(0);
            }
            if self.buf == "kana" {
                self.buf.clear();
                return true;
            }
        }
        false
    }
}

fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let col = branding::centered_column(area, branding::CENTER_MAX_WIDTH);

    let palette_h = if app.palette.is_some() {
        12u16
    } else {
        1u16
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(2),
            Constraint::Min(0),
            Constraint::Length(palette_h),
        ])
        .split(col);

    render_site_header(frame, chunks[0]);

    let tab_labels: Vec<Line> = TABS.iter().map(|t| Line::from(Span::raw(*t))).collect();
    frame.render_widget(
        Tabs::new(tab_labels)
            .select(app.tab)
            .style(Style::default().fg(MUTED))
            .highlight_style(Style::default().fg(ACCENT_STR).add_modifier(Modifier::BOLD))
            .divider("  "),
        chunks[1],
    );

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
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(SURFACE2)),
            )
            .wrap(Wrap { trim: true }),
        chunks[2],
    );

    if let Some(p) = &app.palette {
        let title_line = Line::from(vec![
            Span::styled(" /", Style::default().fg(branding::ACCENT)),
            Span::styled(p.query.as_str(), Style::default().fg(TEXT)),
            Span::styled("▏", Style::default().fg(MUTED)),
        ]);
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(SURFACE2))
            .title(title_line);

        let inner = block.inner(chunks[3]);
        frame.render_widget(block, chunks[3]);

        let inner_split = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(1)])
            .split(inner);

        let pairs = palette_filtered(&p.query);
        let mut lines: Vec<Line> = Vec::new();

        if pairs.is_empty() {
            lines.push(Line::from(Span::styled(
                "no matches",
                Style::default().fg(MUTED),
            )));
        } else {
            for (i, (pal_idx, _, secret)) in pairs.iter().enumerate() {
                let label = PALETTE[*pal_idx].0;
                let is_sel = i == p.selected;
                let prefix = if is_sel { "▸ " } else { "  " };
                let style = if is_sel {
                    Style::default()
                        .fg(ACCENT_STR)
                        .add_modifier(Modifier::BOLD)
                } else if *secret {
                    Style::default().fg(MUTED)
                } else {
                    Style::default().fg(TEXT)
                };
                lines.push(Line::from(vec![
                    Span::styled(prefix, style),
                    Span::styled(label, style),
                ]));
            }
        }

        frame.render_widget(Paragraph::new(lines), inner_split[0]);
        render_footer_hint(
            frame,
            inner_split[1],
            "↑ ↓ select · Enter go · Esc cancel",
        );
    } else {
        render_footer_hint(
            frame,
            chunks[3],
            "← → or h/l · / jump · q quit",
        );
    }
}

pub async fn run(
    handle: Handle,
    channel: ChannelId,
    mut event_rx: mpsc::UnboundedReceiver<TuiEvent>,
    cols: u16,
    rows: u16,
    start: SessionStart,
    channel_mode: ChannelMode,
) -> Result<()> {
    let mut writer = SshWriter::new(handle.clone(), channel);

    execute!(writer, EnterAlternateScreen, cursor::Hide)?;

    let backend = CrosstermBackend::new(writer);
    let mut terminal = Terminal::with_options(
        backend,
        TerminalOptions {
            viewport: Viewport::Fixed(Rect::new(0, 0, cols, rows)),
        },
    )?;

    let mut app = App::new();
    let mut egg = KanaEgg::new();

    if matches!(start, SessionStart::Kana) {
        let mut kana_app = kana::new_app();
        kana::run_session(&mut terminal, &mut event_rx, &mut kana_app).await?;
    }

    terminal.draw(|f| render(f, &app))?;

    loop {
        match event_rx.recv().await {
            None => break,
            Some(TuiEvent::Resize { cols, rows }) => {
                terminal.resize(Rect::new(0, 0, cols, rows))?;
                terminal.draw(|f| render(f, &app))?;
            }
            Some(TuiEvent::Input(data)) => {
                if let Some(ref mut p) = app.palette {
                    match handle_palette_input(&data, p) {
                        PaletteAction::Dismiss => {
                            app.palette = None;
                            terminal.draw(|f| render(f, &app))?;
                        }
                        PaletteAction::Go(goto) => {
                            app.palette = None;
                            match goto {
                                GoTo::Section(i) => app.tab = i,
                                GoTo::Kana => {
                                    let mut kana_app = kana::new_app();
                                    kana::run_session(&mut terminal, &mut event_rx, &mut kana_app)
                                        .await?;
                                }
                            }
                            terminal.draw(|f| render(f, &app))?;
                        }
                        PaletteAction::Stay => {
                            let filtered = palette_filtered(&p.query);
                            if !filtered.is_empty() && p.selected >= filtered.len() {
                                p.selected = filtered.len() - 1;
                            }
                            terminal.draw(|f| render(f, &app))?;
                        }
                    }
                    continue;
                }

                if data == b"/" {
                    app.palette = Some(PaletteState::new());
                    terminal.draw(|f| render(f, &app))?;
                    continue;
                }

                match handle_site_input(&data, &mut app) {
                    SiteAction::Quit => break,
                    SiteAction::Handled => {
                        terminal.draw(|f| render(f, &app))?;
                    }
                    SiteAction::PassThrough => {
                        if egg.feed(&data) {
                            let mut kana_app = kana::new_app();
                            kana::run_session(&mut terminal, &mut event_rx, &mut kana_app).await?;
                            terminal.draw(|f| render(f, &app))?;
                        }
                    }
                }
            }
        }
    }

    execute!(terminal.backend_mut(), LeaveAlternateScreen, cursor::Show)?;
    if matches!(channel_mode, ChannelMode::Exec) {
        let _ = handle.exit_status_request(channel, 0).await;
    }
    let _ = handle.close(channel).await;

    Ok(())
}
