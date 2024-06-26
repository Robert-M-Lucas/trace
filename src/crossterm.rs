use std::{
    error::Error,
    io,
    time::{Duration, Instant},
};
use std::cmp::max;
use ratatui::{
    backend::{Backend, CrosstermBackend},
    crossterm::{
        event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    terminal::Terminal,
};

use crate::{app::App, DATA_TYPE, ui};

pub fn run(tick_rate: Duration, enhanced_graphics: bool, data_countries: DATA_TYPE, data_world: DATA_TYPE) -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let app = App::new("Trace", enhanced_graphics, data_countries, data_world);
    let res = run_app(&mut terminal, app, tick_rate);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    tick_rate: Duration,
) -> io::Result<()> {
    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|f| ui::draw(f, &mut app))?;

        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Esc => app.should_quit = true,
                        KeyCode::Tab => app.show_countries = !app.show_countries,
                        KeyCode::Char('[') => {
                            app.zoom = 1.0f32.max(app.zoom - 1.0);
                        }
                        KeyCode::Char(']') => {
                            app.zoom = 20.0f32.min(app.zoom + 1.0);
                        }
                        KeyCode::Right => {
                            app.map_pos = (app.map_pos.0 + (0.2 / app.zoom), app.map_pos.1);
                        }
                        KeyCode::Left => {
                            app.map_pos = (app.map_pos.0 - (0.2 / app.zoom), app.map_pos.1);
                        }
                        KeyCode::Up => {
                            app.map_pos = (app.map_pos.0, app.map_pos.1 - (0.2 / app.zoom));
                        }
                        KeyCode::Down => {
                            app.map_pos = (app.map_pos.0, app.map_pos.1 + (0.2 / app.zoom));
                        }
                        KeyCode::Backspace => {
                            if !app.input.is_empty() { app.input = app.input.chars().take(app.input.chars().count() - 1).collect(); }
                        }
                        KeyCode::Enter => {
                            app.trace()
                        }
                        KeyCode::Char(c) => app.on_key(c),
                        _ => {}
                    }
                    app.map_pos = (app.map_pos.0.min(1.0).max(0.0), app.map_pos.1.min(1.0).max(0.0));
                }
            }
        }
        if last_tick.elapsed() >= tick_rate {
            app.on_tick();
            last_tick = Instant::now();
        }
        if app.should_quit {
            return Ok(());
        }
    }
}
