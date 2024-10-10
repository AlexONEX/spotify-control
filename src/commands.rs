use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// Changes the service that the DBus commands are sent to
    #[clap(
        short,
        long,
        value_parser,
        default_value = "org.mpris.MediaPlayer2.spotify"
    )]
    pub service_name: String,

    #[clap(subcommand)]
    pub action: Commands,
}

#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum Commands {
    /// Play the next song
    Next,
    /// Play the previous song
    Previous,
    /// Play/Pause the current song
    PlayPause,
    /// Show a notification with the current song
    NowPlaying,
    /// Play a song
    #[clap(name = "play-song")]
    PlaySong {
        #[clap(subcommand)]
        mode: PlayMode,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum PlayMode {
    Uri {
        /// A uri in the format of spotify:track:<id>
        uri: String,
    },
    Search {
        /// You get the best success with "search title artist"
        query: Vec<String>,

        /// Allows picking from a list of songs instead of starting the first
        #[clap(short, long, action)]
        list: bool,

        #[clap(short, long, default_value = "5")]
        count: usize,
    },
}
