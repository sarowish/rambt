mod app;
mod database;
mod musicbrainz;
mod ui;
mod utils;

use crate::app::App;
use crate::ui::render;
use anyhow::Result;
use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::{Backend, CrosstermBackend};
use ratatui::Terminal;
use std::env;
use std::io;
use std::io::{stdin, stdout, Write};
use std::panic;

#[tokio::main]
async fn main() -> Result<()> {
    let mut search_query = String::new();

    if let Some(query) = env::args().nth(1) {
        search_query = query;
    } else {
        print!("Enter artist name: ");
        stdout().flush()?;

        stdin().read_line(&mut search_query).unwrap();
    }

    let mut app = App::new(&search_query)?;

    let default_hook = panic::take_hook();

    panic::set_hook(Box::new(move |info| {
        reset_terminal().unwrap();
        default_hook(info);
    }));

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let res = run_tui(&mut terminal, &mut app);

    reset_terminal()?;

    if let Err(e) = res {
        eprintln!("{e:?}");
    }

    Ok(())
}

fn run_tui<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    loop {
        terminal.clear()?;
        terminal.draw(|f| render(f, app))?;

        if let Event::Key(key) = crossterm::event::read()? {
            match key.code {
                KeyCode::Char('q') => break,
                KeyCode::Char('h') => app.on_left(),
                KeyCode::Char('l') => app.on_right()?,
                KeyCode::Char('j') => app.on_down(),
                KeyCode::Char('k') => app.on_up(),
                KeyCode::Char(c) => match c {
                    rating @ '1'..='9' => app.set_rating(rating.to_digit(10).unwrap() as u8),
                    '0' => app.set_rating(10),
                    _ => (),
                },
                KeyCode::Enter => {
                    if app.currently_rating {
                        app.confirm_rating()?
                    } else if app.releases.is_some() {
                        app.start_rating();
                    }
                }
                KeyCode::Esc => app.abort_rating(),
                _ => (),
            }
        }
    }

    Ok(())
}

fn reset_terminal() -> Result<()> {
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;

    Ok(())
}
