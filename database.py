import sqlite3


def initialize_db(cur: sqlite3.Cursor):
    cur.execute("PRAGMA foreign_keys=ON")

    cur.execute(
        """
            CREATE TABLE IF NOT EXISTS artists (
                artist_id TEXT PRIMARY KEY,
                artist_name TEXT
            )
        """
    )

    cur.execute(
        """
            CREATE TABLE IF NOT EXISTS albums (
                album_id TEXT PRIMARY KEY,
                artist_id TEXT,
                album_name TEXT,
                year INTEGER,
                rating INTEGER,
                FOREIGN KEY(artist_id) REFERENCES artists(artist_id) ON DELETE CASCADE
            )
        """
    )


def add_artist(cur: sqlite3.Cursor, artist_id, artist_name):
    cur.execute(
        """
            INSERT OR IGNORE INTO artists (artist_id, artist_name)
            VALUES(?, ?)
        """,
        (artist_id, artist_name),
    )


def add_album(cur: sqlite3.Cursor, album_id, artist_id, album_name, rating):
    cur.execute(
        """
            INSERT OR REPLACE INTO albums (album_id, artist_id, album_name, rating)
            VALUES(?, ?, ?, ?)

        """,
        (album_id, artist_id, album_name, rating),
    )


def get_ratings(cur: sqlite3.Cursor, artist_id):
    cur.execute(
        """
            SELECT album_id, rating
            FROM albums
            WHERE artist_id=?
        """,
        (artist_id,),
    )

    return cur.fetchall()
