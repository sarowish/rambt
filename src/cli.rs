use clap::{Arg, ArgAction, ArgMatches, Command};

pub fn get_matches() -> ArgMatches {
    Command::new(env!("CARGO_PKG_NAME"))
        .arg(
            Arg::new("rated")
                .short('l')
                .long("list")
                .help("List rated albums")
                .action(ArgAction::SetTrue),
        )
        .arg(Arg::new("artist").value_name("ARTIST"))
        .get_matches()
}
