use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::Frame;

/// Render the interface analysis panel as a popup overlay.
/// `summary_lines` contains pre-formatted interface summary strings.
/// `focus_chain` is the chain index the user has selected (shown as "antibody").
/// `chain_names` lists all chain identifiers in the loaded structure.
pub fn render_interface_panel(
    frame: &mut Frame,
    area: Rect,
    summary_lines: &[String],
    focus_chain: usize,
    chain_names: &[String],
) {
    // --- Compute popup dimensions ---
    // Fixed width of 60 chars, clamped to available space.
    let popup_width = 60u16.min(area.width.saturating_sub(4));
    // Height: 2 (border) + 2 (chain roles) + 1 (hint) + 1 (blank)
    //       + summary_lines.len() + 1 (blank) + 4 (visual key) + 1 (blank)
    //       + 1 (footer) = 13 + summary_lines.len()
    let content_rows = 13u16 + summary_lines.len() as u16;
    let max_height = (area.height * 60) / 100; // ~60% of screen height
    let popup_height = content_rows.min(max_height).min(area.height.saturating_sub(4));

    let x = (area.width.saturating_sub(popup_width)) / 2;
    let y = (area.height.saturating_sub(popup_height)) / 2;
    let popup_area = Rect::new(x, y, popup_width, popup_height);

    // Clear the area behind the popup so underlying content doesn't bleed through.
    frame.render_widget(Clear, popup_area);

    // --- Build the text content ---
    let mut lines: Vec<Line> = Vec::new();

    // -- Chain roles section --
    let focus_name = chain_names
        .get(focus_chain)
        .cloned()
        .unwrap_or_else(|| "?".to_string());

    let other_names: Vec<&String> = chain_names
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != focus_chain)
        .map(|(_, name)| name)
        .collect();
    let other_label = if other_names.is_empty() {
        "-".to_string()
    } else {
        other_names
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<&str>>()
            .join(", ")
    };

    lines.push(Line::from(vec![
        Span::styled("  Focus chain: ", Style::default().fg(Color::White)),
        Span::styled(
            format!("[{}]", focus_name),
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" (antibody)", Style::default().fg(Color::Green)),
    ]));

    lines.push(Line::from(vec![
        Span::styled("  Other chains: ", Style::default().fg(Color::White)),
        Span::styled(
            format!("[{}]", other_label),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" (antigen)", Style::default().fg(Color::Yellow)),
    ]));

    lines.push(Line::from(Span::styled(
        "  Use [ ] to change focus chain",
        Style::default().fg(Color::DarkGray),
    )));

    // Blank separator
    lines.push(Line::from(""));

    // -- Contact summary section --
    for text in summary_lines {
        lines.push(Line::from(Span::styled(
            format!("  {}", text),
            Style::default().fg(Color::White),
        )));
    }

    // Blank separator
    lines.push(Line::from(""));

    // -- Visual key section --
    lines.push(Line::from(Span::styled(
        "  Visual key:",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(vec![
        Span::styled("  Interface residues: ", Style::default().fg(Color::White)),
        Span::styled(
            "BRIGHT",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  Non-interface: ", Style::default().fg(Color::White)),
        Span::styled("dim", Style::default().fg(Color::DarkGray)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  Focus chain: ", Style::default().fg(Color::White)),
        Span::styled("green tones", Style::default().fg(Color::Green)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  Other chains: ", Style::default().fg(Color::White)),
        Span::styled(
            "orange tones",
            Style::default().fg(Color::Rgb(255, 165, 0)),
        ),
    ]));

    // Blank separator
    lines.push(Line::from(""));

    // -- Footer --
    lines.push(Line::from(Span::styled(
        "  Press 'f' to close",
        Style::default().fg(Color::DarkGray),
    )));

    // --- Build the paragraph widget with bordered block ---
    let panel = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(" Interface Analysis "),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(panel, popup_area);
}
