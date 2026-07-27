use crate::app::App;
use crate::github::PrKind;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Clear, List, ListItem, ListState, Paragraph, Row, Table, TableState},
    Frame,
};

pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(5),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(f.area());

    // PR Table
    let header = Row::new(vec![
        Cell::from("Kind"),
        Cell::from("Repo"),
        Cell::from("#"),
        Cell::from("Title"),
        Cell::from("Labels"),
        Cell::from("Author"),
        Cell::from("Updated"),
    ])
    .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));

    let filtered = app.filtered_prs();

    let rows: Vec<Row> = filtered
        .iter()
        .map(|pr| {
            let kind_cell = Cell::from(pr.kind.label()).style(match pr.kind {
                PrKind::Review => Style::default().fg(Color::Magenta),
                PrKind::Assignee => Style::default().fg(Color::Blue),
            });
            Row::new(vec![
                kind_cell,
                Cell::from(pr.repo().to_string()),
                Cell::from(format!("#{}", pr.number)),
                Cell::from(pr.title.clone()),
                Cell::from(Span::styled(pr.labels_str(), Style::default().fg(Color::Cyan))),
                Cell::from(pr.author().to_string()),
                Cell::from(pr.updated_short().to_string()),
            ])
        })
        .collect();

    let widths = [
        Constraint::Length(10),
        Constraint::Percentage(15),
        Constraint::Length(7),
        Constraint::Percentage(30),
        Constraint::Percentage(15),
        Constraint::Percentage(10),
        Constraint::Percentage(12),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Review Requests & Assigned PRs "),
        )
        .row_highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    let mut state = TableState::default();
    if !filtered.is_empty() {
        state.select(Some(app.selected));
    }
    f.render_stateful_widget(table, chunks[0], &mut state);

    // Status bar
    let status = if let Some(ref err) = app.error {
        Line::from(vec![
            Span::styled("ERROR: ", Style::default().fg(Color::Red)),
            Span::raw(err.as_str()),
        ])
    } else if app.refreshing {
        Line::from(vec![
            Span::styled(
                " Refreshing...",
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!(
                " | {}/{} PRs | Filter: {} | Last updated: {}",
                filtered.len(), app.prs.len(), app.filter.label(), app.last_updated
            )),
        ])
    } else {
        Line::from(vec![
            Span::styled(
                format!(" {}/{} PRs", filtered.len(), app.prs.len()),
                Style::default().fg(Color::Green),
            ),
            Span::raw(format!(
                " | Filter: {} | Last updated: {}",
                app.filter.label(), app.last_updated
            )),
        ])
    };
    f.render_widget(Paragraph::new(status), chunks[1]);

    // Keybinds help
    let help = Line::from(vec![
        Span::styled(" q", Style::default().fg(Color::Cyan)),
        Span::raw(":quit "),
        Span::styled("j/k", Style::default().fg(Color::Cyan)),
        Span::raw(":move "),
        Span::styled("Enter", Style::default().fg(Color::Cyan)),
        Span::raw(":open "),
        Span::styled("r", Style::default().fg(Color::Cyan)),
        Span::raw(":refresh "),
        Span::styled("Tab", Style::default().fg(Color::Cyan)),
        Span::raw(":filter "),
        Span::styled("a", Style::default().fg(Color::Cyan)),
        Span::raw(":actions"),
    ]);
    f.render_widget(Paragraph::new(help), chunks[2]);

    // Manual-action popup menu (opened with `a`).
    if app.show_action_menu && !app.action_names.is_empty() {
        let items: Vec<ListItem> = app
            .action_names
            .iter()
            .map(|n| ListItem::new(n.clone()))
            .collect();
        // Size the popup to the content, centered.
        let width = app
            .action_names
            .iter()
            .map(|n| n.len())
            .max()
            .unwrap_or(10)
            .clamp(12, 50) as u16
            + 6;
        let height = (app.action_names.len() as u16).min(15) + 2;
        let area = centered_rect(width, height, f.area());
        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Actions (Enter:run  Esc:close) "),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("> ");
        let mut state = ListState::default();
        state.select(Some(app.action_selected));
        f.render_widget(Clear, area); // clear the area behind the popup
        f.render_stateful_widget(list, area, &mut state);
    }
}

/// A rectangle of the given width/height centered within `area`.
fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let w = width.min(area.width);
    let h = height.min(area.height);
    let x = area.x + (area.width.saturating_sub(w)) / 2;
    let y = area.y + (area.height.saturating_sub(h)) / 2;
    Rect { x, y, width: w, height: h }
}
