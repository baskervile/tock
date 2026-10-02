use crate::gradient::GradientBox;
use crate::state::{AppMode, EditMode};
use figlet_rs::FIGlet;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Paragraph, Row, Table, TableState},
};

pub struct UiData<'a> {
    pub font: &'a FIGlet,
    pub time_str: &'a str,
    pub subtitle_str: &'a str,
    pub footer_str: &'a str,
    pub bg_color: Color,
    pub clock_color: Color,
    pub subtitle_color: Color,
    pub gradient: Option<GradientBox>,
    pub hidden_help: bool,
    pub show_panel: bool,
    pub panel_ratio: u8,
    pub panel_bg: Color,
    pub panel_fg: Color,
    pub panel_border: Color,
    pub panel_border_sides: Borders,
    pub panel_border_style: BorderType,
    pub mode: AppMode,
    pub items: &'a [Vec<String>],
    pub headers: &'a [&'static str],
    pub table_state: &'a mut TableState,
    pub edit_mode: &'a EditMode,
    pub input_buffer: &'a str,
}

fn build_layout(
    area: Rect,
    show_panel: bool,
    panel_ratio: u8,
    content_height: usize,
) -> (Rect, Option<Rect>) {
    if show_panel {
        let panel_ratio = panel_ratio.clamp(1, 99) as u16;
        let outer_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(100 - panel_ratio),
                Constraint::Percentage(panel_ratio),
            ])
            .split(area);

        let content_container = outer_chunks[0];
        let panel_area = outer_chunks[1];

        let container_height = content_container.height as usize;

        let content_area = if container_height <= content_height {
            content_container
        } else {
            let remaining = container_height - content_height;
            let top_space = remaining / 2;

            let inner_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(top_space as u16),
                    Constraint::Length(content_height as u16),
                    Constraint::Length((remaining - top_space) as u16),
                ])
                .split(content_container);
            inner_chunks[1]
        };
        (content_area, Some(panel_area))
    } else {
        let available_height = area.height as usize;
        let actual_content_height = content_height.min(available_height);
        let top_space = (available_height - actual_content_height) / 2;

        let content_area = Rect::new(
            area.x,
            area.y + top_space as u16,
            area.width,
            actual_content_height as u16,
        );
        (content_area, None)
    }
}

fn max_line_width(text: &str) -> usize {
    text.lines()
        .map(|line| line.chars().count())
        .max()
        .unwrap_or(0)
}

fn pad_ascii_lines(ascii_art: &str, fixed_width: usize) -> Vec<String> {
    ascii_art
        .lines()
        .map(|line| {
            let line_width = line.chars().count();
            if line_width >= fixed_width {
                line.to_string()
            } else {
                format!("{}{}", line, " ".repeat(fixed_width - line_width))
            }
        })
        .collect()
}

fn figlet_lines(font: &FIGlet, value: &str) -> Vec<String> {
    font.convert(value)
        .map(|figure| figure.to_string().lines().map(str::to_string).collect())
        .unwrap_or_default()
}

fn figlet_height(font: &FIGlet) -> usize {
    figlet_lines(font, "0").len().max(1)
}

fn digit_cell_width(font: &FIGlet) -> usize {
    ('0'..='9')
        .map(|digit| max_line_width(&figlet_lines(font, &digit.to_string()).join("\n")))
        .max()
        .unwrap_or(0)
}

fn blank_lines(height: usize, width: usize) -> Vec<String> {
    vec![" ".repeat(width); height]
}

fn normalize_glyph(mut lines: Vec<String>, height: usize, width: usize) -> Vec<String> {
    lines.resize_with(height, String::new);
    lines
        .into_iter()
        .take(height)
        .map(|line| {
            let line_width = line.chars().count();
            if line_width >= width {
                line
            } else {
                format!("{}{}", line, " ".repeat(width - line_width))
            }
        })
        .collect()
}

fn fixed_width_time_lines(font: &FIGlet, value: &str) -> Vec<String> {
    const DIGIT_GAP: usize = 1;
    const SPACE_WIDTH: usize = 2;

    let height = figlet_height(font);
    let digit_width = digit_cell_width(font);
    let mut rows = vec![String::new(); height];
    let mut previous_was_digit = false;

    for ch in value.chars() {
        let glyph = if ch.is_ascii_digit() {
            if previous_was_digit {
                for row in &mut rows {
                    row.push_str(&" ".repeat(DIGIT_GAP));
                }
            }
            normalize_glyph(figlet_lines(font, &ch.to_string()), height, digit_width)
        } else if ch == ' ' {
            blank_lines(height, SPACE_WIDTH)
        } else {
            let glyph_lines = figlet_lines(font, &ch.to_string());
            let width = max_line_width(&glyph_lines.join("\n"));
            normalize_glyph(glyph_lines, height, width)
        };

        for (row, glyph_line) in rows.iter_mut().zip(glyph) {
            row.push_str(&glyph_line);
        }

        previous_was_digit = ch.is_ascii_digit();
    }

    rows
}

fn centered_fixed_area(area: Rect, width: usize) -> Rect {
    let width = width.min(area.width as usize) as u16;
    let x = area.x + area.width.saturating_sub(width) / 2;

    Rect::new(x, area.y, width, area.height)
}

pub fn draw(f: &mut Frame, data: &mut UiData) {
    let area = f.area();
    if area.width == 0 || area.height == 0 {
        return;
    }

    // 1. Draw global background
    f.render_widget(
        Block::default().style(Style::default().bg(data.bg_color)),
        area,
    );

    // 2. Vertical layout - footer always at bottom (1 row)
    let footer_height = 1;
    let main_area_height = area.height.saturating_sub(footer_height);
    let main_area = Rect::new(area.x, area.y, area.width, main_area_height);

    // 3. Generate ASCII art and calculate content height
    let mut ascii_lines = fixed_width_time_lines(data.font, data.time_str);
    if max_line_width(&ascii_lines.join("\n")) > main_area.width as usize
        || ascii_lines.len() + 2 > main_area.height as usize
    {
        ascii_lines = vec![data.time_str.replace(" : ", ":").replace(" . ", ".")];
    }
    let fixed_width = ascii_lines
        .iter()
        .map(|line| line.chars().count())
        .max()
        .unwrap_or(0);
    let ascii_lines = pad_ascii_lines(&ascii_lines.join("\n"), fixed_width);
    let ascii_height = ascii_lines.len();
    let subtitle_height = data.subtitle_str.lines().count();
    let content_height = ascii_height + subtitle_height + 1;

    let (content_area, panel_area) =
        build_layout(main_area, data.show_panel, data.panel_ratio, content_height);
    let ascii_area = Rect::new(
        content_area.x,
        content_area.y,
        content_area.width,
        ascii_height as u16,
    );
    let time_area = centered_fixed_area(ascii_area, fixed_width);
    let subtitle_area = Rect::new(
        content_area.x,
        content_area.y + ascii_height as u16,
        content_area.width,
        subtitle_height as u16,
    );

    // 4. Draw center area (ASCII art + Subtitle)
    if let Some(ref gradient) = data.gradient {
        let max_width = fixed_width.max(1);

        let mut lines = Vec::new();

        for line in &ascii_lines {
            let mut spans = Vec::new();
            for (j, ch) in line.chars().enumerate() {
                let ratio = j as f32 / max_width.saturating_sub(1).max(1) as f32;
                let color = gradient.at(ratio).to_rgba8();
                let ratatui_color = Color::Rgb(color[0], color[1], color[2]);
                spans.push(Span::styled(
                    ch.to_string(),
                    Style::default().fg(ratatui_color),
                ));
            }
            lines.push(Line::from(spans));
        }

        let time_paragraph = Paragraph::new(Text::from(lines)).alignment(Alignment::Left);

        f.render_widget(time_paragraph, time_area);
    } else {
        let mut lines = Vec::new();

        for line in &ascii_lines {
            lines.push(Line::from(Span::styled(
                line,
                Style::default().fg(data.clock_color),
            )));
        }

        let time_paragraph = Paragraph::new(Text::from(lines)).alignment(Alignment::Left);

        f.render_widget(time_paragraph, time_area);
    }

    let subtitle_paragraph = Paragraph::new(data.subtitle_str)
        .style(Style::default().fg(data.subtitle_color))
        .alignment(Alignment::Center);
    f.render_widget(subtitle_paragraph, subtitle_area);

    if let Some(panel_area) = panel_area {
        let panel_block = Block::default()
            .title(data.mode.title())
            .title_alignment(Alignment::Center)
            .borders(data.panel_border_sides)
            .border_type(data.panel_border_style)
            .style(Style::default().fg(data.panel_fg).bg(data.panel_bg))
            .border_style(Style::default().fg(data.panel_border));
        f.render_widget(panel_block, panel_area);

        draw_table(f, panel_area, data);
    }

    // 6. Draw footer hints (always at bottom)
    if !data.hidden_help {
        let footer_area = Rect::new(area.x, area.y + area.height - 1, area.width, 1);
        let footer_paragraph = Paragraph::new(data.footer_str)
            .style(Style::default().fg(Color::DarkGray).bg(data.bg_color))
            .alignment(Alignment::Center);

        f.render_widget(footer_paragraph, footer_area);
    }
}

fn draw_table(f: &mut Frame, area: Rect, data: &mut UiData) {
    if area.width < 3 || area.height < 3 {
        return;
    }
    let inner = Rect::new(
        area.x + 1,
        area.y + 1,
        area.width.saturating_sub(2),
        area.height.saturating_sub(2),
    );

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(inner);

    let header = Row::new(
        data.headers
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>(),
    )
    .style(Style::new().bold())
    .bottom_margin(1);

    let (sel_r, sel_c) = (
        data.table_state.selected(),
        data.table_state.selected_column(),
    );
    let is_typing = matches!(data.edit_mode, EditMode::Typing { .. });

    let col_count = data.headers.len();
    let widths: Vec<Constraint> = (0..col_count)
        .map(|_| Constraint::Percentage(100 / col_count as u16))
        .collect();

    let rows: Vec<Row> = data
        .items
        .iter()
        .enumerate()
        .map(|(r_idx, item)| {
            let cells: Vec<String> = item
                .iter()
                .enumerate()
                .map(|(c_idx, s)| {
                    if is_typing && Some(r_idx) == sel_r && Some(c_idx) == sel_c {
                        format!("{}█", data.input_buffer)
                    } else {
                        s.to_string()
                    }
                })
                .collect();
            Row::new(cells)
        })
        .collect();

    let cell_style = if is_typing {
        Style::new().reversed().light_blue()
    } else {
        Style::new().reversed().dark_gray()
    };

    let table = Table::new(rows, widths)
        .header(header)
        .column_spacing(1)
        .style(Color::White)
        .row_highlight_style(Style::new().on_black().bold())
        .cell_highlight_style(cell_style)
        .highlight_symbol("❱ ");

    f.render_stateful_widget(table, chunks[0], data.table_state);

    if !data.hidden_help {
        let help_text = match data.edit_mode {
            EditMode::Normal => {
                if data.mode == AppMode::Stopwatch {
                    " 'd' Delete | ↑↓←→ Navigate | g/G First/Last "
                } else {
                    " 'a' Add | 'e' Edit | 'd' Delete | ↑↓←→ Navigate | g/G First/Last "
                }
            }
            EditMode::Typing { is_new_row: true } => {
                " [Adding] Enter: next field | Esc: abort | Space: toggle "
            }
            EditMode::Typing { is_new_row: false } => {
                " [Editing] Enter: save | Esc: cancel | Space: toggle "
            }
        };

        let help_widget = Paragraph::new(help_text)
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);

        f.render_widget(help_widget, chunks[1]);
    }
}

#[cfg(test)]
mod recovery_tests {
    use super::*;
    #[test]
    fn changing_digits_preserves_timer_width() {
        for name in ["ANSI_Shadow", "3d", "3D-ASCII"] {
            let font = FIGlet::from_file(&format!("assets/{name}.flf")).unwrap();
            for samples in [
                ["00 : 00 : 00", "11 : 11 : 11", "88 : 88 : 88"],
                [
                    "00 : 00 : 00 . 000",
                    "11 : 11 : 11 . 111",
                    "88 : 88 : 88 . 888",
                ],
            ] {
                let widths: Vec<_> = samples
                    .iter()
                    .map(|s| max_line_width(&fixed_width_time_lines(&font, s).join("\n")))
                    .collect();
                assert!(widths.iter().all(|w| *w == widths[0]), "{name}: {widths:?}");
            }
        }
    }
    #[test]
    fn timer_area_centers_and_clips() {
        assert_eq!(
            centered_fixed_area(Rect::new(10, 5, 100, 8), 40),
            Rect::new(40, 5, 40, 8)
        );
        assert_eq!(
            centered_fixed_area(Rect::new(10, 5, 20, 8), 40),
            Rect::new(10, 5, 20, 8)
        );
    }
}

#[cfg(test)]
mod rendering_tests {
    use super::*;
    use ratatui::{Terminal, backend::TestBackend};
    fn render(width: u16, height: u16, panel: bool) -> Terminal<TestBackend> {
        let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
        let font = FIGlet::standard().unwrap();
        let mut state = TableState::default();
        terminal
            .draw(|frame| {
                draw(
                    frame,
                    &mut UiData {
                        font: &font,
                        time_str: "12 : 34 : 56",
                        subtitle_str: "Friday",
                        footer_str: "q quit",
                        bg_color: Color::Reset,
                        clock_color: Color::Cyan,
                        subtitle_color: Color::Reset,
                        gradient: crate::gradient::parse_color(
                            "warm",
                            &std::collections::HashMap::new(),
                        ),
                        hidden_help: false,
                        show_panel: panel,
                        panel_ratio: 50,
                        panel_bg: Color::Reset,
                        panel_fg: Color::Cyan,
                        panel_border: Color::Cyan,
                        panel_border_sides: Borders::ALL,
                        panel_border_style: BorderType::Rounded,
                        mode: AppMode::Clock,
                        items: &[],
                        headers: &["Time"],
                        table_state: &mut state,
                        edit_mode: &EditMode::Normal,
                        input_buffer: "",
                    },
                )
            })
            .unwrap();
        terminal
    }
    #[test]
    fn rgb_gradient_reaches_the_terminal_buffer() {
        let t = render(160, 30, false);
        let colors: std::collections::HashSet<_> = t
            .backend()
            .buffer()
            .content
            .iter()
            .filter_map(|cell| {
                if let Color::Rgb(r, g, b) = cell.fg {
                    Some((r, g, b))
                } else {
                    None
                }
            })
            .collect();
        assert!(colors.len() > 10);
    }
    #[test]
    fn tiny_windows_and_panels_do_not_panic() {
        for (w, h) in [(0, 0), (1, 1), (2, 2), (10, 3), (40, 10)] {
            render(w, h, true);
            render(w, h, false);
        }
    }
}
