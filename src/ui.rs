use crate::{
    app::{App, ListItemType},
    musicbrainz::ArtistSearchResult,
};
use ratatui::{
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState},
    Frame,
};

pub struct StatefulList<T> {
    state: ListState,
    pub items: Vec<T>,
}

impl<T> StatefulList<T> {
    pub fn with_items(items: Vec<T>) -> StatefulList<T> {
        let mut list = StatefulList {
            state: ListState::default(),
            items,
        };

        if !list.items.is_empty() {
            list.state.select(Some(0));
        }

        list
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn get_selected(&self) -> Option<&T> {
        match self.state.selected() {
            Some(i) => Some(&self.items[i]),
            None => None,
        }
    }

    pub fn get_mut_selected(&mut self) -> Option<&mut T> {
        match self.state.selected() {
            Some(i) => Some(&mut self.items[i]),
            None => None,
        }
    }
}

pub fn render(f: &mut Frame, app: &mut App) {
    if app.releases.is_some() {
        render_releases(f, app);
    } else if let Some(results) = &mut app.search_results {
        render_search_results(f, results);
    } else {
        render_ratings(f, app);
    }
}

pub fn render_search_results(f: &mut Frame, results: &mut StatefulList<ArtistSearchResult>) {
    let artists = results
        .items
        .iter()
        .map(|ar| ar.to_string())
        .map(Span::raw)
        .map(ListItem::new)
        .collect::<Vec<ListItem>>();

    let artists = List::new(artists)
        .highlight_symbol("> ")
        .highlight_style(
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title("Releases")
                .title_style(Style::default().cyan().bold()),
        );

    f.render_stateful_widget(artists, f.area(), &mut results.state);
}

pub fn render_releases(f: &mut Frame, app: &mut App) {
    let Some(releases) = &mut app.releases else {
        return;
    };

    let selected_index = releases.state.selected().unwrap();
    let mut list_items = Vec::new();

    for (idx, release) in releases.items.iter().enumerate() {
        let item = match release {
            ListItemType::ReleaseType(r#type) => vec![Span::styled(
                r#type.to_string(),
                Style::default().fg(Color::Green),
            )],
            ListItemType::Release(release) => rating_line(
                &release.to_string(),
                release.rating,
                idx == selected_index,
                app.currently_rating,
            ),
        };

        list_items.push(ListItem::new(Line::from(item)));
    }

    let list = List::new(list_items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title("Releases")
            .title_style(Style::default().cyan().bold()),
    );
    f.render_stateful_widget(list, f.area(), &mut releases.state);
}

pub fn render_ratings(f: &mut Frame, app: &mut App) {
    let Some(rated) = &mut app.rated_list else {
        return;
    };

    let selected_index = rated.state.selected().unwrap();
    let mut list_items = Vec::new();

    for (idx, item) in rated.items.iter().enumerate() {
        list_items.push(ListItem::new(Line::from(rating_line(
            &item.to_string(),
            item.rating,
            idx == selected_index,
            app.currently_rating,
        ))));
    }

    let list = List::new(list_items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title("Releases")
            .title_style(Style::default().fg(Color::Cyan).bold()),
    );
    f.render_stateful_widget(list, f.area(), &mut rated.state);
}

pub fn rating_line(
    line: &str,
    rating: Option<u8>,
    selected: bool,
    currently_rating: bool,
) -> Vec<Span<'static>> {
    let mut stars = String::new();
    let mut stars_filler = String::new();

    if let Some(rating) = rating {
        stars = "★ ".repeat(rating as usize / 2);
        stars.push_str(&"⯨".repeat(rating as usize % 2));

        stars_filler = "⯩".repeat(rating as usize % 2);
        stars_filler.push_str(&"★ ".repeat((10 - stars.chars().count()) / 2));
    }

    vec![
        Span::styled(format!("{} {} ", if selected { ">" } else { " " }, line), {
            let mut style = Style::default();

            if selected {
                style = style
                    .fg(if currently_rating {
                        Color::Blue
                    } else {
                        Color::Magenta
                    })
                    .bold();
            }

            style
        }),
        Span::styled(
            stars,
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(stars_filler, Style::default().bold()),
    ]
}
