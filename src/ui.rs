use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    terminal::Frame,
    text::{self, Span},
    widgets::{
        canvas::{self, Canvas, Circle, Map, MapResolution, Rectangle},
        Axis, BarChart, Block, Cell, Chart, Dataset, Gauge, LineGauge, List, ListItem, Paragraph,
        Row, Sparkline, Table, Tabs, Wrap,
    },
};
use ratatui::style::Stylize;
use crate::app::App;
use crate::conv_coords;
use crate::custom_map::CMap;

pub fn draw(f: &mut Frame, app: &mut App) {
    let chunks = Layout::vertical([Constraint::Length(3), Constraint::Min(0)]).split(f.size());
    let tabs = app
        .tabs
        .titles
        .iter()
        .map(|t| text::Line::from(Span::styled(*t, Style::default().fg(Color::Green))))
        .collect::<Tabs>()
        .block(Block::bordered().title(app.title))
        .highlight_style(Style::default().fg(Color::Yellow))
        .select(app.tabs.index);
    f.render_widget(tabs, chunks[0]);
    match app.tabs.index {
        0 => draw_first_tab(f, app, chunks[1]),
        _ => {}
    };
}


fn draw_text(f: &mut Frame, area: Rect) {
    let text = vec![
        text::Line::from("This is a paragraph with several lines. You can change style your text the way you want"),
        text::Line::from(""),
        text::Line::from(vec![
            Span::from("For example: "),
            Span::styled("under", Style::default().fg(Color::Red)),
            Span::raw(" "),
            Span::styled("the", Style::default().fg(Color::Green)),
            Span::raw(" "),
            Span::styled("rainbow", Style::default().fg(Color::Blue)),
            Span::raw("."),
        ]),
        text::Line::from(vec![
            Span::raw("Oh and if you didn't "),
            Span::styled("notice", Style::default().add_modifier(Modifier::ITALIC)),
            Span::raw(" you can "),
            Span::styled("automatically", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" "),
            Span::styled("wrap", Style::default().add_modifier(Modifier::REVERSED)),
            Span::raw(" your "),
            Span::styled("text", Style::default().add_modifier(Modifier::UNDERLINED)),
            Span::raw(".")
        ]),
        text::Line::from(
            "One more thing is that it should display unicode characters: 10€"
        ),
    ];
    let block = Block::bordered().title(Span::styled(
        "Footer",
        Style::default()
            .fg(Color::Magenta)
            .add_modifier(Modifier::BOLD),
    ));
    let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: true });
    f.render_widget(paragraph, area);
}

fn draw_first_tab(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks =
        Layout::horizontal([Constraint::Percentage(30), Constraint::Percentage(70)]).split(area);

    let h_chunks = Layout::vertical([Constraint::Length(3), Constraint::Fill(1), Constraint::Length(3)]).split(chunks[0]);

    let table = Table::new(
        [Row::new(vec![format!("> {}", app.input)]).style(Style::default().bold())],
        [
            Constraint::Min(0),
        ],
    )
    .block(Block::bordered().title("Input"));
    f.render_widget(table, h_chunks[0]);

    let rows = app.trace_result.iter().enumerate().map(|(_, t)| {
        Row::new(vec![t.no.clone(), t.ip.clone(), t.name.clone(), t.time.clone()]).style(Style::default())
    });
    let table = Table::new(
        rows,
        [
            Constraint::Length(5),
            Constraint::Length(16),
            Constraint::Fill(1),
            Constraint::Length(10),
        ],
    )
    .header(
        Row::new(vec!["No.", "IP", "Name", "Time"])
            .style(Style::default().fg(Color::Yellow))
            .bottom_margin(1),
    )
    .block(Block::bordered().title("Servers"));
    f.render_widget(table, h_chunks[1]);

    let table = Table::new(
        [Row::new(vec![format!(" {}", app.status.clone())]).style(
            if app.error {
                Style::default().red().bold()
            }
            else {
                Style::default().green().bold()
            }
        )],
        [
            Constraint::Min(0),
        ],
    )
        .block(Block::bordered().title("Status"));
    f.render_widget(table, h_chunks[2]);

    let map = Canvas::default()
        .block(Block::bordered().title("World - TAB to enable borders"))
        .paint(|ctx| {
            ctx.draw(&CMap {
                data: if app.show_countries { app.data_countries.clone() } else { app.data_world.clone() },
                pos: app.map_pos,
                zoom: app.zoom
            });
            ctx.layer();
            for (i, s1) in app.trace_result.iter().enumerate().filter(|(_, x)| !x.lat.is_nan()) {
                let Some(s2) = app.trace_result.iter().skip(i + 1).filter(|x| !x.lat.is_nan()).next() else { break; };

                let (x1, y1) = conv_coords(s1.long, s1.lat, app.zoom, app.map_pos);
                let (x2, y2) = conv_coords(s2.long, s2.lat, app.zoom, app.map_pos);

                let (x1, y1, x2, y2) = constrain(x1, y1, x2, y2);

                ctx.draw(&canvas::Line {
                    x1: x1 as f64,
                    y1: y1 as f64,
                    x2: x2 as f64,
                    y2: y2 as f64,
                    color: Color::Yellow,
                });
            }

            for s in &app.trace_result {
                let (x1, y1) = conv_coords(s.long, s.lat, app.zoom, app.map_pos);
                ctx.print(
                    x1 as f64,
                    y1 as f64,
                    Span::styled("X", Style::default().green()),
                );
            }
        })
        .marker(if app.enhanced_graphics {
            symbols::Marker::Braille
        } else {
            symbols::Marker::Dot
        })
        .x_bounds([0.0, 1.0])
        .y_bounds([0.0, 1.0]);
    f.render_widget(map, chunks[1]);
}

fn constrain(mut x1: f32, mut y1: f32, mut x2: f32, mut y2: f32) -> (f32, f32, f32, f32) {
    if x2 == x1 || y1 == y2 {
        return (x1, y1, x2, y2)
    }
    if x1 == x2 {
        return (x1, y1.max(0.0).min(1.0), x2, y2.max(0.0).min(1.0));
    }
    if y1 == y2 {
        return (x1.max(0.0).min(1.0), y1, x2.max(0.0).min(1.0), y2);
    }
    let gradient = (y2 - y1) / (x2 - x1);
    let inv_grad = 1.0 / gradient;

    if x1 < 0.0 || y1 < 0.0 {
        let y_int = y1 - (gradient * x1);
        let x_int = x1 - (inv_grad * y1);
        if x_int > y_int {
            x1 = x_int;
            y1 = 0.0;
        }
        else {
            y1 = y_int;
            x1 = 0.0;
        }
    }

    if x1 > 1.0 || y1 > 1.0 {
        let y_int = y1 - (gradient * (x1 + 1.0));
        let x_int = x1 - (inv_grad * (y1 + 1.0));
        if x_int < y_int {
            x1 = x_int;
            y1 = 1.0;
        }
        else {
            y1 = y_int;
            x1 = 1.0;
        }
    }

    let gradient = (y1 - y2) / (x1 - x2);
    let inv_grad = 1.0 / gradient;

    if x2 < 0.0 || y2 < 0.0 {
        let y_int = y2 - (gradient * x2);
        let x_int = x2 - (inv_grad * y2);
        if x_int > y_int {
            x2 = x_int;
            y2 = 0.0;
        }
        else {
            y2 = y_int;
            x2 = 0.0;
        }
    }

    if x2 > 1.0 || y2 > 1.0 {
        let y_int = y2 - (gradient * (x2 + 1.0));
        let x_int = x2 - (inv_grad * (y2 + 1.0));
        if x_int < y_int {
            x2 = x_int;
            y2 = 1.0;
        }
        else {
            y2 = y_int;
            x2 = 1.0;
        }
    }

    (x1, y1, x2, y2)
}