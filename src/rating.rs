use std::fmt::Display;

pub trait Rate {
    fn rating(&mut self) -> &mut Option<u8>;

    fn set_rating(&mut self, rating: u8) {
        *self.rating() = Some(rating);
    }

    fn increase_rating(&mut self) {
        if let Some(rating) = self.rating() {
            if *rating != 10 {
                *rating += 1;
            }
        }
    }

    fn decrease_rating(&mut self) {
        if let Some(rating) = self.rating() {
            if *rating != 1 {
                *rating -= 1;
            }
        }
    }
}

pub struct Rated {
    pub artist_id: String,
    pub artist_name: String,
    pub release_id: String,
    pub title: String,
    pub rating: Option<u8>,
}

impl Rate for Rated {
    fn rating(&mut self) -> &mut Option<u8> {
        &mut self.rating
    }
}

impl TryFrom<&rusqlite::Row<'_>> for Rated {
    type Error = rusqlite::Error;

    fn try_from(row: &rusqlite::Row) -> Result<Self, Self::Error> {
        Ok(Rated {
            artist_id: row.get(0)?,
            artist_name: row.get(1)?,
            release_id: row.get(2)?,
            title: row.get(3)?,
            rating: row.get(4)?,
        })
    }
}

impl Display for Rated {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} - {}", self.artist_name, self.title)
    }
}
