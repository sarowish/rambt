use crate::{database, musicbrainz::*, ui::StatefulList, utils::get_database_path};
use anyhow::Result;
use futures::executor;
use musicbrainz_rs::entity::release_group::{ReleaseGroupPrimaryType, ReleaseGroupSecondaryType};
use rusqlite::Connection;
use serde::Serialize;
use std::fmt::Display;

#[derive(PartialEq, Eq, Clone)]
pub struct ReleaseType {
    primary: Option<ReleaseGroupPrimaryType>,
    secondary: Vec<ReleaseGroupSecondaryType>,
}

impl ReleaseType {
    pub fn new(
        primary: Option<ReleaseGroupPrimaryType>,
        secondary: Vec<ReleaseGroupSecondaryType>,
    ) -> Self {
        Self { primary, secondary }
    }
}

impl Display for ReleaseType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut result = Vec::new();

        if let Some(primary) = &self.primary {
            result.push(release_type_to_string(primary));
        }

        result.extend(self.secondary.iter().map(release_type_to_string));

        write!(f, "{}", result.join(" + "))
    }
}

fn release_type_to_string<T: Serialize>(value: &T) -> String {
    serde_json::to_string(value)
        .unwrap()
        .trim_matches('"')
        .to_string()
}

pub enum ListItemType {
    ReleaseType(ReleaseType),
    Release(Release),
}

pub struct App {
    pub search_results: StatefulList<ArtistSearchResult>,
    pub releases: Option<StatefulList<ListItemType>>,
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
        if let Some(releases) = &self.releases {
            if let Some(ListItemType::Release(release)) = releases.get_selected() {
                return release;
            }
        }

        unreachable!();
    }

    fn get_mut_selected_release(&mut self) -> &mut Release {
        if let Some(releases) = &mut self.releases {
            if let Some(ListItemType::Release(release)) = releases.get_mut_selected() {
                return release;
            }
        }

        unreachable!();
    }

    pub fn on_down(&mut self) {
        if self.currently_rating {
            return;
        }

        if let Some(releases) = &mut self.releases {
            releases.next();

            if let Some(ListItemType::ReleaseType(_)) = releases.get_selected() {
                releases.next();
            }
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

            if let Some(ListItemType::ReleaseType(_)) = releases.get_selected() {
                releases.previous();
            }
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
                if let Ok(Some(mut releases)) = executor::block_on(fetch_releases(&artist.id)) {
                    let ratings = database::get_ratings(&self.conn, &artist.id)?;

                    for rating in ratings {
                        for release in &mut releases {
                            if release.id == rating.0 {
                                release.rating = Some(rating.1);
                            }
                        }
                    }

                    self.releases = Some(StatefulList::with_items(insert_headers(releases)));
                    self.releases.as_mut().unwrap().next();
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
        let release = self.get_selected_release();

        database::add_artist(&self.conn, &artist.id, &artist.name)?;
        database::add_release(&self.conn, &artist.id, release)?;

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

fn insert_headers(releases: Vec<Release>) -> Vec<ListItemType> {
    let mut releases = releases.into_iter();
    let mut result = if let Some(release) = releases.next() {
        vec![
            ListItemType::ReleaseType(release.group_type.clone()),
            ListItemType::Release(release),
        ]
    } else {
        return Vec::new();
    };

    for release in releases {
        if matches!(result.last(), Some(ListItemType::Release(last_item)) if release.group_type != last_item.group_type)
        {
            result.push(ListItemType::ReleaseType(release.group_type.clone()));
        }

        result.push(ListItemType::Release(release));
    }

    result
}
