use anyhow::Result;
use rand::seq::SliceRandom;
use rand::Rng;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame, Terminal,
};
use tokio::sync::mpsc;

use crate::branding::{
    self, render_footer_hint, render_site_header, ACCENT, ACCENT_STR, MUTED, SURFACE2, TEXT,
};
use crate::content;
use crate::events::TuiEvent;
use crate::ssh_writer::SshWriter;

const ERR: Color = Color::Rgb(204, 36, 29);

struct KanaEntry {
    character: &'static str,
    accepted: &'static [&'static str],
}

fn kana_list() -> Vec<KanaEntry> {
    vec![
        KanaEntry { character: "あ", accepted: &["a"] },
        KanaEntry { character: "い", accepted: &["i"] },
        KanaEntry { character: "う", accepted: &["u"] },
        KanaEntry { character: "え", accepted: &["e"] },
        KanaEntry { character: "お", accepted: &["o"] },
        KanaEntry { character: "か", accepted: &["ka"] },
        KanaEntry { character: "き", accepted: &["ki"] },
        KanaEntry { character: "く", accepted: &["ku"] },
        KanaEntry { character: "け", accepted: &["ke"] },
        KanaEntry { character: "こ", accepted: &["ko"] },
        KanaEntry { character: "さ", accepted: &["sa"] },
        KanaEntry { character: "し", accepted: &["shi", "si"] },
        KanaEntry { character: "す", accepted: &["su"] },
        KanaEntry { character: "せ", accepted: &["se"] },
        KanaEntry { character: "そ", accepted: &["so"] },
        KanaEntry { character: "た", accepted: &["ta"] },
        KanaEntry { character: "ち", accepted: &["chi", "ti"] },
        KanaEntry { character: "つ", accepted: &["tsu", "tu"] },
        KanaEntry { character: "て", accepted: &["te"] },
        KanaEntry { character: "と", accepted: &["to"] },
        KanaEntry { character: "な", accepted: &["na"] },
        KanaEntry { character: "に", accepted: &["ni"] },
        KanaEntry { character: "ぬ", accepted: &["nu"] },
        KanaEntry { character: "ね", accepted: &["ne"] },
        KanaEntry { character: "の", accepted: &["no"] },
        KanaEntry { character: "は", accepted: &["ha"] },
        KanaEntry { character: "ひ", accepted: &["hi"] },
        KanaEntry { character: "ふ", accepted: &["fu", "hu"] },
        KanaEntry { character: "へ", accepted: &["he"] },
        KanaEntry { character: "ほ", accepted: &["ho"] },
        KanaEntry { character: "ま", accepted: &["ma"] },
        KanaEntry { character: "み", accepted: &["mi"] },
        KanaEntry { character: "む", accepted: &["mu"] },
        KanaEntry { character: "め", accepted: &["me"] },
        KanaEntry { character: "も", accepted: &["mo"] },
        KanaEntry { character: "や", accepted: &["ya"] },
        KanaEntry { character: "ゆ", accepted: &["yu"] },
        KanaEntry { character: "よ", accepted: &["yo"] },
        KanaEntry { character: "ら", accepted: &["ra"] },
        KanaEntry { character: "り", accepted: &["ri"] },
        KanaEntry { character: "る", accepted: &["ru"] },
        KanaEntry { character: "れ", accepted: &["re"] },
        KanaEntry { character: "ろ", accepted: &["ro"] },
        KanaEntry { character: "わ", accepted: &["wa"] },
        KanaEntry { character: "を", accepted: &["wo"] },
        KanaEntry { character: "ん", accepted: &["n", "nn"] },
        KanaEntry { character: "ア", accepted: &["a"] },
        KanaEntry { character: "イ", accepted: &["i"] },
        KanaEntry { character: "ウ", accepted: &["u"] },
        KanaEntry { character: "エ", accepted: &["e"] },
        KanaEntry { character: "オ", accepted: &["o"] },
        KanaEntry { character: "カ", accepted: &["ka"] },
        KanaEntry { character: "キ", accepted: &["ki"] },
        KanaEntry { character: "ク", accepted: &["ku"] },
        KanaEntry { character: "ケ", accepted: &["ke"] },
        KanaEntry { character: "コ", accepted: &["ko"] },
        KanaEntry { character: "サ", accepted: &["sa"] },
        KanaEntry { character: "シ", accepted: &["shi", "si"] },
        KanaEntry { character: "ス", accepted: &["su"] },
        KanaEntry { character: "セ", accepted: &["se"] },
        KanaEntry { character: "ソ", accepted: &["so"] },
        KanaEntry { character: "タ", accepted: &["ta"] },
        KanaEntry { character: "チ", accepted: &["chi", "ti"] },
        KanaEntry { character: "ツ", accepted: &["tsu", "tu"] },
        KanaEntry { character: "テ", accepted: &["te"] },
        KanaEntry { character: "ト", accepted: &["to"] },
        KanaEntry { character: "ナ", accepted: &["na"] },
        KanaEntry { character: "ニ", accepted: &["ni"] },
        KanaEntry { character: "ヌ", accepted: &["nu"] },
        KanaEntry { character: "ネ", accepted: &["ne"] },
        KanaEntry { character: "ノ", accepted: &["no"] },
        KanaEntry { character: "ハ", accepted: &["ha"] },
        KanaEntry { character: "ヒ", accepted: &["hi"] },
        KanaEntry { character: "フ", accepted: &["fu", "hu"] },
        KanaEntry { character: "ヘ", accepted: &["he"] },
        KanaEntry { character: "ホ", accepted: &["ho"] },
        KanaEntry { character: "マ", accepted: &["ma"] },
        KanaEntry { character: "ミ", accepted: &["mi"] },
        KanaEntry { character: "ム", accepted: &["mu"] },
        KanaEntry { character: "メ", accepted: &["me"] },
        KanaEntry { character: "モ", accepted: &["mo"] },
        KanaEntry { character: "ヤ", accepted: &["ya"] },
        KanaEntry { character: "ユ", accepted: &["yu"] },
        KanaEntry { character: "ヨ", accepted: &["yo"] },
        KanaEntry { character: "ラ", accepted: &["ra"] },
        KanaEntry { character: "リ", accepted: &["ri"] },
        KanaEntry { character: "ル", accepted: &["ru"] },
        KanaEntry { character: "レ", accepted: &["re"] },
        KanaEntry { character: "ロ", accepted: &["ro"] },
        KanaEntry { character: "ワ", accepted: &["wa"] },
        KanaEntry { character: "ヲ", accepted: &["wo"] },
        KanaEntry { character: "ン", accepted: &["n", "nn"] },
    ]
}

enum Feedback {
    None,
    Correct,
    Wrong { correct: String },
}

pub(crate) struct KanaApp {
    kana: Vec<KanaEntry>,
    remaining: Vec<usize>,
    current: Option<usize>,
    input: String,
    feedback: Feedback,
    total: usize,
    show_info: bool,
    /// Shown until the user interacts with the input once; never comes back this session.
    show_input_placeholder: bool,
}

impl KanaApp {
    fn new() -> Self {
        let kana = kana_list();
        let total = kana.len();
        let mut remaining: Vec<usize> = (0..total).collect();
        remaining.shuffle(&mut rand::thread_rng());
        let current = remaining.last().copied();
        Self {
            kana,
            remaining,
            current,
            input: String::new(),
            feedback: Feedback::None,
            total,
            show_info: false,
            show_input_placeholder: true,
        }
    }

    fn completed(&self) -> usize {
        self.total - self.remaining.len()
    }

    fn is_done(&self) -> bool {
        self.remaining.is_empty()
    }

    fn submit(&mut self) {
        self.show_input_placeholder = false;
        if self.input.is_empty() {
            return;
        }
        let Some(idx) = self.current else {
            return;
        };

        let answer = self.input.trim().to_lowercase();
        let entry = &self.kana[idx];

        if entry.accepted.iter().any(|&a| a == answer) {
            self.remaining.pop();
            self.feedback = Feedback::Correct;
        } else {
            let correct = entry.accepted[0].to_string();
            self.feedback = Feedback::Wrong { correct };

            if self.remaining.len() > 1 {
                let last = self.remaining.len() - 1;
                let item = self.remaining.pop().unwrap();
                let pos = rand::thread_rng().gen_range(0..last);
                self.remaining.insert(pos, item);
            }
        }

        self.input.clear();
        self.current = self.remaining.last().copied();
    }
}

pub(crate) enum KanaAction {
    /// Leave kana and return to the main TUI.
    Quit,
    /// Stay in kana; caller should redraw.
    Continue,
}

fn handle_kana_input(data: &[u8], app: &mut KanaApp) -> KanaAction {
    if app.is_done() {
        return KanaAction::Quit;
    }

    if app.show_info {
        if data == b"\x1b" || data == b"\r" || data == b"\n" {
            app.show_info = false;
            return KanaAction::Continue;
        }
        if data.len() == 1 && matches!(data[0], b'I' | b'q' | b'Q') {
            app.show_info = false;
            return KanaAction::Continue;
        }
        return KanaAction::Continue;
    }

    if data.len() == 1 {
        match data[0] {
            b'I' => {
                app.show_info = true;
                return KanaAction::Continue;
            }
            // Lone ESC — not an arrow / other CSI sequence.
            0x1b | b'q' | b'Q' | 3 => return KanaAction::Quit,
            b'\r' | b'\n' => {
                app.submit();
                return KanaAction::Continue;
            }
            0x7f | 0x08 => {
                app.show_input_placeholder = false;
                app.input.pop();
                return KanaAction::Continue;
            }
            b if b.is_ascii_graphic() => {
                app.show_input_placeholder = false;
                if !matches!(app.feedback, Feedback::None) {
                    app.feedback = Feedback::None;
                }
                app.input.push(b as char);
                return KanaAction::Continue;
            }
            _ => {}
        }
    }
    KanaAction::Continue
}

fn render_kana(frame: &mut Frame, app: &KanaApp) {
    let area = frame.area();
    let col = branding::centered_column(area, branding::CENTER_MAX_WIDTH);

    if app.is_done() {
        let outer = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(2),
                Constraint::Min(0),
                Constraint::Length(1),
            ])
            .split(col);

        render_site_header(frame, outer[0]);

        let done_title = format!(" Progress: {}/{} ", app.total, app.total);
        let text = vec![
            Line::from(""),
            Line::from(Span::styled(
                "All kana mastered!",
                Style::default()
                    .fg(ACCENT_STR)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                format!("You answered all {} characters correctly.", app.total),
                Style::default().fg(TEXT),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Press any key · Esc or q",
                Style::default().fg(MUTED),
            )),
        ];
        frame.render_widget(
            Paragraph::new(text)
                .alignment(Alignment::Center)
                .style(Style::default().fg(TEXT))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(SURFACE2))
                        .title(Span::styled(done_title, Style::default().fg(ACCENT))),
                ),
                outer[1],
        );
        render_footer_hint(
            frame,
            outer[2],
            "Enter · Backspace · Esc or q to return",
        );
        return;
    }

    // Inner width ≈ column minus borders/margins (for wrapping the wrong-answer line).
    let inner_w = col.width.saturating_sub(6).max(8) as usize;
    let inner_rows: u16 = {
        let mut n = 0u16;
        if matches!(app.feedback, Feedback::None) || !app.input.is_empty() {
            n += 1;
        }
        match &app.feedback {
            Feedback::Correct => n += 1,
            Feedback::Wrong { correct } => {
                let msg = format!("Wrong — try again. Answer: {}", correct);
                n += ((msg.len().max(1) + inner_w - 1) / inner_w) as u16;
            }
            Feedback::None => {}
        }
        n.max(1)
    };
    let input_box_h = inner_rows.saturating_add(2);

    let outer = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(2),
            Constraint::Min(0),
            Constraint::Length(input_box_h),
            Constraint::Length(1),
        ])
        .split(col);

    render_site_header(frame, outer[0]);

    let completed = app.completed();
    let progress_title = format!(" Progress: {}/{} ", completed, app.total);
    let glyph_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(SURFACE2))
        .title(Span::styled(progress_title, Style::default().fg(ACCENT)));

    let glyph_inner = glyph_block.inner(outer[1]);
    frame.render_widget(glyph_block, outer[1]);

    if let Some(idx) = app.current {
        let k = &app.kana[idx];
        let inner_h = glyph_inner.height as usize;
        let top_pad = inner_h.saturating_sub(2) / 2;
        let padding = "\n".repeat(top_pad.saturating_sub(1));
        let display = format!("{}{}", padding, k.character);
        frame.render_widget(
            Paragraph::new(display)
                .alignment(Alignment::Center)
                .style(
                    Style::default()
                        .fg(ACCENT_STR)
                        .add_modifier(Modifier::BOLD),
                ),
            glyph_inner,
        );
    }

    let input_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(SURFACE2));

    let input_inner = input_block.inner(outer[2]);
    frame.render_widget(input_block, outer[2]);

    let input_style = match &app.feedback {
        Feedback::Correct => Style::default().fg(ACCENT_STR),
        Feedback::Wrong { .. } => Style::default().fg(ERR),
        Feedback::None => Style::default().fg(TEXT),
    };

    let input_line = if app.show_input_placeholder && app.input.is_empty() {
        Line::from(Span::styled(
            "Enter kana romaji",
            Style::default().fg(MUTED),
        ))
    } else {
        Line::from(Span::styled(
            if app.input.is_empty() {
                " "
            } else {
                app.input.as_str()
            },
            input_style,
        ))
    };

    let mut input_lines = Vec::new();
    if matches!(app.feedback, Feedback::None) || !app.input.is_empty() {
        input_lines.push(input_line);
    }
    match &app.feedback {
        Feedback::Correct => input_lines.push(Line::from(Span::styled(
            "Correct!",
            Style::default()
                .fg(ACCENT_STR)
                .add_modifier(Modifier::BOLD),
        ))),
        Feedback::Wrong { correct } => input_lines.push(Line::from(Span::styled(
            format!("Wrong — try again. Answer: {}", correct),
            Style::default().fg(ERR).add_modifier(Modifier::BOLD),
        ))),
        Feedback::None => {}
    }

    frame.render_widget(
        Paragraph::new(input_lines).wrap(Wrap { trim: true }),
        input_inner,
    );

    let footer = if app.show_info {
        "Esc or Shift+I to close info"
    } else {
        "Enter · Backspace · Shift+I info · Esc or q to return"
    };
    render_footer_hint(frame, outer[3], footer);

    if app.show_info {
        let body = Rect {
            y: outer[1].y,
            height: outer[1].height.saturating_add(outer[2].height),
            ..outer[1]
        };
        render_kana_info_overlay(frame, col, body);
    }
}

fn render_kana_info_overlay(frame: &mut Frame, col: Rect, body_area: Rect) {
    let w = (col.width * 4 / 5).max(42).min(col.width);
    let h = body_area.height.saturating_sub(2).max(10).min(18);
    let x = col.x + col.width.saturating_sub(w) / 2;
    let y = body_area.y + body_area.height.saturating_sub(h) / 2;
    let popup = Rect::new(x, y, w, h);

    frame.render_widget(Clear, popup);
    frame.render_widget(
        Paragraph::new(content::KANA_INFO)
            .style(Style::default().fg(TEXT))
            .wrap(Wrap { trim: true })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(SURFACE2))
                    .title(Span::styled(
                        " kana · how to open ",
                        Style::default().fg(ACCENT),
                    )),
            ),
        popup,
    );
}

pub(crate) async fn run_session(
    terminal: &mut Terminal<CrosstermBackend<SshWriter>>,
    event_rx: &mut mpsc::UnboundedReceiver<TuiEvent>,
    app: &mut KanaApp,
) -> Result<()> {
    terminal.draw(|f| render_kana(f, app))?;

    loop {
        match event_rx.recv().await {
            None => break,
            Some(TuiEvent::Resize { cols, rows }) => {
                terminal.resize(Rect::new(0, 0, cols, rows))?;
                terminal.draw(|f| render_kana(f, app))?;
            }
            Some(TuiEvent::Input(data)) => {
                if matches!(handle_kana_input(&data, app), KanaAction::Quit) {
                    break;
                }
                terminal.draw(|f| render_kana(f, app))?;
            }
        }
    }

    Ok(())
}

pub(crate) fn new_app() -> KanaApp {
    KanaApp::new()
}
