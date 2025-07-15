use crate::{app::ReleaseType, rating::Rate};
use musicbrainz_rs::{
    chrono::Datelike,
    entity::{
        artist::{Artist, ArtistSearchQuery},
        release_group::ReleaseGroup,
    },
    prelude::*,
};
use std::fmt::Display;

pub struct ArtistSearchResult {
    pub id: String,
    pub name: String,
    pub disambiguation: String,
}

impl Display for ArtistSearchResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.name, self.disambiguation)
    }
}

impl From<&Artist> for ArtistSearchResult {
    fn from(value: &Artist) -> Self {
        ArtistSearchResult {
            id: value.id.to_string(),
            name: value.name.to_string(),
            disambiguation: value.disambiguation.to_string(),
        }
    }
}

#[derive(Default)]
pub struct Release {
    pub id: String,
    pub title: String,
    pub year: i32,
    pub group_type: ReleaseType,
    pub rating: Option<u8>,
}

impl Rate for Release {
    fn rating(&mut self) -> &mut Option<u8> {
        &mut self.rating
    }
}

impl Display for Release {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}) {}", self.year, self.title)
    }
}

impl From<ReleaseGroup> for Release {
    fn from(value: ReleaseGroup) -> Self {
        Release {
            id: value.id.to_string(),
            title: value.title.to_string(),
            year: value
                .first_release_date
                .and_then(|date| date.into_naive_date(1, 1, 1).ok())
                .unwrap_or_default()
                .year(),
            group_type: ReleaseType::new(value.primary_type, value.secondary_types),
            rating: None,
        }
    }
}

pub async fn search_artist(artist_name: &str) -> Result<Vec<ArtistSearchResult>, Error> {
    let query = ArtistSearchQuery::query_builder()
        .artist(artist_name)
        .build();

    let query_result = Artist::search(query).execute().await?;

    Ok(query_result
        .entities
        .iter()
        .map(ArtistSearchResult::from)
        .collect::<Vec<ArtistSearchResult>>())
}

pub async fn fetch_releases(artist_id: &str) -> Result<Option<Vec<Release>>, Error> {
    let artist = Artist::fetch()
        .with_release_groups()
        .id(artist_id)
        .execute()
        .await?;

    Ok(artist.release_groups.map(|release_groups| {
        release_groups
            .into_iter()
            .map(Release::from)
            .collect::<Vec<Release>>()
    }))
}
