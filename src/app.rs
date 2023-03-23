use crate::{database, musicbrainz::*, ui::StatefulList, utils::get_database_path};
use anyhow::Result;
use futures::executor;
use rusqlite::Connection;

pub struct App {
    pub search_results: StatefulList<ArtistSearchResult>,
    pub releases: Option<StatefulList<Release>>,
    pub currently_rating: bool,
    previous_rating: Option<u8>,
    conn: Connection,
}

impl App {
    pub fn new(search_query: &str) -> Result<Self> {
        let search_results = executor::block_on(search_artist(search_query)).unwrap_or_default();

        if search_results.is_empty() {
            println!("No artists were found with the given query.");
            std::process::exit(1);
        }

        let app = App {
            search_results: StatefulList::with_items(search_results),
            releases: None,
            currently_rating: false,
            previous_rating: None,
            conn: Connection::open(get_database_path()?)?,
        };

        database::initialize_db(&app.conn)?;

        Ok(app)
    }

    fn get_selected_release(&self) -> &Release {
        self.releases.as_ref().unwrap().get_selected().unwrap()
    }

    fn get_mut_selected_release(&mut self) -> &mut Release {
        self.releases.as_mut().unwrap().get_mut_selected().unwrap()
    }

    pub fn on_down(&mut self) {
        if self.currently_rating {
            return;
        }

        if let Some(releases) = &mut self.releases {
            releases.next();
        } else {
            self.search_results.next();
        }
    }

    pub fn on_up(&mut self) {
        if self.currently_rating {
            return;
        }

        if let Some(releases) = &mut self.releases {
            releases.previous();
        } else {
            self.search_results.previous();
        }
    }

    pub fn on_left(&mut self) {
        if self.currently_rating {
            self.get_mut_selected_release().decrease_rating();
        } else if self.releases.is_some() {
            std::mem::take(&mut self.releases);
        }
    }

    pub fn on_right(&mut self) -> Result<()> {
        if self.currently_rating {
            self.get_mut_selected_release().increase_rating();
        } else if self.releases.is_none() {
            if let Some(artist) = self.search_results.get_selected() {
                if let Ok(Some(release)) = executor::block_on(fetch_releases(&artist.id)) {
                    self.releases = Some(StatefulList::with_items(release));

                    let ratings = database::get_ratings(&self.conn, &artist.id)?;

                    for rating in ratings {
                        for release in &mut self.releases.as_mut().unwrap().items {
                            if release.id == rating.0 {
                                release.rating = Some(rating.1);
                            }
                        }
                    }
                }
            }
        } else {
            self.start_rating();
        }

        Ok(())
    }

    pub fn start_rating(&mut self) {
        self.currently_rating = true;
        self.previous_rating = self.get_selected_release().rating;
        let selected_release = self.get_mut_selected_release();

        if selected_release.rating.is_none() {
            selected_release.rating = Some(1);
        }
    }

    pub fn set_rating(&mut self, rating: u8) {
        if self.currently_rating {
            self.get_mut_selected_release().rating = Some(rating);
        }
    }

    pub fn confirm_rating(&mut self) -> Result<()> {
        if !self.currently_rating {
            return Ok(());
        }

        let artist = self.search_results.get_selected().unwrap();
        let release = self.releases.as_ref().unwrap().get_selected().unwrap();

        database::add_artist(&self.conn, &artist.id, &artist.name)?;
        database::add_release(
            &self.conn,
            &artist.id,
            &release.id,
            &release.title,
            release.rating.unwrap(),
        )?;

        self.currently_rating = false;

        Ok(())
    }

    pub fn abort_rating(&mut self) {
        if !self.currently_rating {
            return;
        }

        self.currently_rating = false;
        self.get_mut_selected_release().rating = self.previous_rating;
    }
}
