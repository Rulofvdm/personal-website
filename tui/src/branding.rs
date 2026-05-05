//! Shared colours and layout for the site TUI and embedded views (e.g. kana).

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

pub const TEXT: Color = Color::Rgb(235, 219, 178);
pub const MUTED: Color = Color::Rgb(146, 131, 116);
pub const ACCENT: Color = Color::Rgb(93, 138, 93);
pub const ACCENT_STR: Color = Color::Rgb(169, 182, 101);
pub const SURFACE2: Color = Color::Rgb(60, 56, 54);

pub const CENTER_MAX_WIDTH: u16 = 90;

pub fn centered_column(area: Rect, max_width: u16) -> Rect {
    let width = max_width.min(area.width);
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    Rect { x, width, ..area }
}

pub fn render_site_header(frame: &mut Frame, area: Rect) {
    frame.render_widget(
        Paragraph::new(vec![
            Line::from(Span::styled(
                "rulof van der merwe",
                Style::default().fg(TEXT).add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
                "─────────────────────",
                Style::default().fg(ACCENT),
            )),
        ]),
        area,
    );
}

pub fn render_footer_hint(frame: &mut Frame, area: Rect, hint: &str) {
    frame.render_widget(
        Paragraph::new(Span::styled(hint, Style::default().fg(MUTED))),
        area,
    );
}
