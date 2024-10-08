use crate::error::Error;
use crate::metadata::Metadata;
use crate::spotify_api::search;
use clap::{Parser, Subcommand};
use notify_rust::{Hint, Notification};
use std::collections::HashMap;
use std::io::Write;
use zbus::dbus_proxy;
use zbus::zvariant::OwnedValue;

#[dbus_proxy(
    interface = "org.mpris.MediaPlayer2.Player",
    default_path = "/org/mpris/MediaPlayer2"
)]
trait PlayerProxy {
    fn play_pause(&self) -> zbus::Result<()>;
    fn next(&self) -> zbus::Result<()>;
    fn previous(&self) -> zbus::Result<()>;
    fn open_uri(&self, uri: &str) -> zbus::Result<()>;
    #[dbus_proxy(property)]
    fn metadata(&self) -> zbus::Result<HashMap<String, OwnedValue>>;
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

pub async fn what(metadata: Metadata) -> Result<(), Error> {
    let res = reqwest::get(&metadata.artwork).await?;
    let bytes = res.bytes().await?;
    let tmp = temp_file::with_contents(&bytes);
    Notification::new()
        .appname("Spotify Notify")
        .summary(&metadata.title)
        .body(&format!(
            "{} - {}",
            metadata.artists.join(", "),
            metadata.album
        ))
        .image_path(tmp.path().to_str().unwrap())
        .hint(Hint::Category("music".to_string()))
        .show()?;
    Ok(())
}

pub async fn play_song(proxy: &PlayerProxy, mode: PlayMode) -> Result<(), Error> {
    match mode {
        PlayMode::Uri { uri } => {
            proxy.open_uri(&uri).await?;
            Ok(())
        }
        PlayMode::Search { query, list, count } => {
            let query = query.join(" ");
            let tracks = search(&query).await?;
            if list {
                for (i, track) in tracks.iter().take(count).enumerate() {
                    println!("{} - {}", i, track);
                }
                print!("Enter a number to play: ");
                std::io::stdout().flush()?;
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                let input = input.trim().parse::<usize>().unwrap();
                if let Some(track) = tracks.get(input) {
                    println!("Playing {}", track);
                    let uri = format!("spotify:track:{}", track.id);
                    proxy.open_uri(&uri).await?;
                } else {
                    println!("Invalid selection");
                }
            } else if let Some(track) = tracks.first() {
                println!("Playing {}", track);
                let uri = format!("spotify:track:{}", track.id);
                proxy.open_uri(&uri).await?;
            } else {
                println!("No track found for {}", query);
            }
            Ok(())
        }
    }
}
