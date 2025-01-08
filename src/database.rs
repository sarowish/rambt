use crate::musicbrainz::Release;
use anyhow::Result;
use rusqlite::{params, Connection};

pub fn initialize_db(conn: &Connection) -> Result<()> {
    conn.pragma_update(None, "foreign_keys", "on")?;

    conn.execute(
        "
            CREATE TABLE IF NOT EXISTS artists (
                artist_id TEXT PRIMARY KEY,
                artist_name TEXT
            )
        ",
        [],
    )?;

    conn.execute(
        "
            CREATE TABLE IF NOT EXISTS releases (
                artist_id TEXT,
                release_id TEXT PRIMARY KEY,
                release_name TEXT,
                year INTEGER,
                rating INTEGER,
                FOREIGN KEY(artist_id) REFERENCES artists(artist_id) ON DELETE CASCADE
            )
        ",
        [],
    )?;

    Ok(())
}

pub fn add_artist(conn: &Connection, artist_id: &str, artist_name: &str) -> Result<()> {
    conn.execute(
        "
            INSERT OR IGNORE INTO artists (artist_id, artist_name)
            VALUES(?1, ?2)
        ",
        params![artist_id, artist_name],
    )?;

    Ok(())
}

pub fn add_release(conn: &Connection, artist_id: &str, release: &Release) -> Result<()> {
    conn.execute(
        "
            INSERT OR REPLACE INTO releases (artist_id, release_id, release_name, rating)
            VALUES(?, ?, ?, ?)

        ",
        params![artist_id, release.id, release.title, release.rating],
    )?;

    Ok(())
}

pub fn get_ratings(conn: &Connection, artist_id: &str) -> Result<Vec<(String, u8)>> {
    let mut stmt = conn.prepare(
        "
            SELECT release_id, rating
            FROM releases
            WHERE artist_id=?1
        ",
    )?;

    let mut ratings = Vec::new();

    for rating in stmt.query_map(params![artist_id], |row| Ok((row.get(0)?, row.get(1)?)))? {
        ratings.push(rating?);
    }

    Ok(ratings)
}
