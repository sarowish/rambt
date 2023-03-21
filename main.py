from enum import Enum
import platformdirs
import curses
import musicbrainzngs
import os
import database
import sqlite3

APP_NAME = "rambt"


class StatefulList:
    def __init__(self, items):
        self.selected = 0
        self.items = items

    def go_up(self):
        if self.selected == 0:
            self.selected = len(self.items) - 1
        else:
            self.selected -= 1

    def go_down(self):
        if self.selected == len(self.items) - 1:
            self.selected = 0
        else:
            self.selected += 1

    def draw_list(self, screen):
        for i, item in enumerate(self.items):
            item.draw(screen, i, i == self.selected)

    def get_selected(self):
        return self.items[self.selected]


class Artist:
    def __init__(self, id, name, disambiguation):
        self.id = id
        self.name = name
        self.disambiguation = disambiguation

    def draw(self, screen, y, selected):
        attributes = 0

        if selected:
            attributes = curses.color_pair(1) | curses.A_BOLD
            screen.addch(y, 0, ">", attributes)

        screen.addstr(
            y, 2, "{} ({})".format(self.name, self.disambiguation), attributes
        )


class Album:
    def __init__(self, id, title, release_date):
        self.id = id
        self.title = title
        self.year = release_date.split("-")[0]
        self.rating = None
        self.being_rated = False

    def draw(self, screen, y, selected):
        rating = []
        rating_filler = []

        if self.rating is not None:
            rating = ["★ "] * (self.rating // 2)
            rating += ["⯨"] * (self.rating % 2)
            rating_filler += ["⯩"] * ((self.rating) % 2)
            rating_filler += ["★ "] * (5 - len(rating))

        rating = "".join(rating)
        rating_filler = "".join(rating_filler)

        attributes = 0

        if selected:
            if self.being_rated:
                attributes = curses.color_pair(2)
            else:
                attributes = curses.color_pair(1)

            attributes |= curses.A_BOLD

            screen.addch(y, 0, ">", attributes)

        screen.addstr(y, 2, f"({self.year}) {self.title}", attributes)
        screen.addstr("  ")

        screen.addstr(rating, curses.color_pair(3) | curses.A_BOLD)
        screen.addstr(rating_filler, curses.A_BOLD)

    def increase_rating(self):
        if self.rating != 10:
            self.rating += 1

    def decrease_rating(self):
        if self.rating != 1:
            self.rating -= 1


Window = Enum("Window", "ARTISTS ALBUMS")


class App:
    def __init__(self, screen):
        musicbrainzngs.set_useragent(APP_NAME, "0.1")

        database_path = os.path.join(get_data_dir(), "ratings.db")

        self.conn = sqlite3.connect(database_path)
        self.cur = self.conn.cursor()
        database.initialize_db(self.cur)

        self.screen = screen
        self.album = None

    def get_selected_list(self):
        if self.window == Window.ARTISTS:
            return self.artists
        else:
            return self.albums

    def confirm_rating(self):
        artist = self.artists.get_selected()

        database.add_artist(self.cur, artist.id, artist.name)
        database.add_album(
            self.cur, self.album.id, artist.id, self.album.title, self.album.rating
        )
        self.conn.commit()

    def handle_key(self, c):
        if self.window == Window.ARTISTS:
            if c == ord("l"):
                self.window = Window.ALBUMS
                self.pick_artist()

            return

        if self.album is None:
            if c == ord("l") or c == ord("\n"):
                self.album = self.albums.get_selected()
                self.album.being_rated = True
                self.previous_rating = self.album.rating

                if self.album.rating is None:
                    self.album.rating = 1
            elif c == ord("h") or c == curses.KEY_LEFT:
                self.album = None
                self.window = Window.ARTISTS
        else:
            if c == ord("\n"):
                self.confirm_rating()
                self.album.being_rated = False
                self.album = None
            elif c == 27:
                self.album.rating = self.previous_rating
                self.album.being_rated = False
                self.album = None
            elif c == ord("0"):
                self.album.rating = 10
            elif 49 <= c <= 57:
                self.album.rating = c - 48
            elif c == ord("h") or c == curses.KEY_LEFT:
                self.album.decrease_rating()
            elif c == ord("l") or c == curses.KEY_RIGHT:
                self.album.increase_rating()

    def search_artists(self):
        self.screen.addstr("Enter artist name: ")
        curses.echo()
        artist = self.screen.getstr()
        curses.noecho()

        search_results = musicbrainzngs.search_artists(artist=artist)["artist-list"]

        self.artists = StatefulList(
            [
                Artist(artist["id"], artist["name"], artist.get("disambiguation", ""))
                for artist in search_results
            ]
        )

        self.window = Window.ARTISTS

    def pick_artist(self):
        artist_id = self.artists.get_selected().id

        releases = musicbrainzngs.get_artist_by_id(
            artist_id, includes=["release-groups"]
        )["artist"]["release-group-list"]

        self.albums = StatefulList(
            [
                Album(album["id"], album["title"], album["first-release-date"])
                for album in list(filter(is_album, releases))
            ]
        )

        ratings = database.get_ratings(self.cur, artist_id)

        for rating in ratings:
            for album in self.albums.items:
                if album.id == rating[0]:
                    album.rating = rating[1]


def is_album(release):
    return release["type"] == "Album"


def get_data_dir():
    path = platformdirs.user_data_dir(APP_NAME)

    if not os.path.exists(path):
        os.mkdir(path)

    return path


def wrapper(func, /, *args, **kwds):
    try:
        screen = curses.initscr()

        curses.noecho()
        curses.cbreak()

        screen.keypad(True)

        curses.start_color()
        curses.use_default_colors()
        curses.init_pair(1, curses.COLOR_MAGENTA, -1)
        curses.init_pair(2, curses.COLOR_BLUE, -1)
        curses.init_pair(3, curses.COLOR_YELLOW, -1)

        return func(screen, *args, **kwds)
    finally:
        if "screen" in locals():
            reset_terminal(screen)


def reset_terminal(screen):
    screen.keypad(False)
    curses.echo()
    curses.nocbreak()
    curses.endwin()


def main(screen):
    app = App(screen)

    app.search_artists()
    curses.curs_set(0)

    while True:
        screen.clear()

        selected_list = app.get_selected_list()
        selected_list.draw_list(screen)

        c = screen.getch()

        if app.album is not None:
            app.handle_key(c)
        elif c == ord("j") or c == curses.KEY_DOWN:
            selected_list.go_down()
        elif c == ord("k") or c == curses.KEY_UP:
            selected_list.go_up()
        elif c == ord("q"):
            reset_terminal(screen)
            break
        else:
            app.handle_key(c)


wrapper(main)
