mod app;
mod cli;
mod database;
mod musicbrainz;
mod rating;
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
use std::io;
use std::io::{stdin, stdout, Write};
use std::panic;

#[tokio::main]
async fn main() -> Result<()> {
    let clap_args = cli::get_matches();

    let mut app = if clap_args.get_flag("rated") {
        App::list_rated()?
    } else if let Some(query) = clap_args.get_one::<String>("artist") {
        App::search(query)?
    } else {
        print!("Enter artist name: ");
        stdout().flush()?;

        let mut search_query = String::new();
        stdin().read_line(&mut search_query).unwrap();
        App::search(&search_query)?
    };

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
        terminal.draw(|f| render(f, app))?;

        if let Event::Key(key) = crossterm::event::read()? {
            match key.code {
                KeyCode::Char('q') => break,
                KeyCode::Char('h') | KeyCode::Left => app.on_left(),
                KeyCode::Char('l') | KeyCode::Right => app.on_right()?,
                KeyCode::Char('j') | KeyCode::Down => app.on_down(),
                KeyCode::Char('k') | KeyCode::Up => app.on_up(),
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
