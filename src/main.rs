use std::{collections::HashMap, fmt::Display, io::Write};

use clap::{Parser, Subcommand};
use notify_rust::{Hint, Notification};
use serde::{Deserialize, Serialize};
use zbus::{
    dbus_proxy,
    names::{BusName, Error as NamesError},
    zvariant::{Array, OwnedValue},
};

#[dbus_proxy(
    interface = "org.mpris.MediaPlayer2.Player",
    default_path = "/org/mpris/MediaPlayer2"
)]
trait Player {
    fn play_pause(&self) -> zbus::Result<()>;
    fn next(&self) -> zbus::Result<()>;
    fn previous(&self) -> zbus::Result<()>;
    fn open_uri(&self, uri: &str) -> zbus::Result<()>;
    #[dbus_proxy(property)]
    fn metadata(&self) -> zbus::Result<HashMap<String, OwnedValue>>;
}

#[derive(Debug)]
pub enum Error {
    ZbusError(zbus::Error),
    ZbusNamesError(NamesError),
    MetadataError(MetadataError),
    ReqwestError(reqwest::Error),
    NotificationError(notify_rust::error::Error),
    IoError(std::io::Error),
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::ZbusNamesError(e) => write!(f, "DBus names error: {}", e),
            Error::ZbusError(e) => write!(f, "DBus error: {}", e),
            Error::MetadataError(e) => write!(f, "Metadata error: {}", e),
            Error::ReqwestError(e) => write!(f, "HTTP request error: {}", e),
            Error::NotificationError(e) => write!(f, "Notification error: {}", e),
            Error::IoError(e) => write!(f, "I/O error: {}", e),
        }
    }
}

impl From<zbus::Error> for Error {
    fn from(err: zbus::Error) -> Self {
        Error::ZbusError(err)
    }
}

impl From<NamesError> for Error {
    fn from(err: NamesError) -> Self {
        Error::ZbusNamesError(err)
    }
}

impl From<MetadataError> for Error {
    fn from(err: MetadataError) -> Self {
        Error::MetadataError(err)
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::ReqwestError(err)
    }
}

impl From<notify_rust::error::Error> for Error {
    fn from(err: notify_rust::error::Error) -> Self {
        Error::NotificationError(err)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IoError(err)
    }
}

#[derive(Debug, Clone)]
pub enum MetadataError {
    MissingKey(String),
    InvalidValueType(String),
}

impl std::error::Error for MetadataError {}

impl Display for MetadataError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MetadataError::MissingKey(key) => write!(f, "Missing metadata key: {}", key),
            MetadataError::InvalidValueType(key) => {
                write!(f, "Invalid value type for key: {}", key)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Metadata {
    title: String,
    artists: Vec<String>,
    album: String,
    artwork: String,
}

impl TryFrom<HashMap<String, OwnedValue>> for Metadata {
    type Error = MetadataError;

    fn try_from(map: HashMap<String, OwnedValue>) -> Result<Self, Self::Error> {
        Ok(Metadata {
            title: get_string(&map, "xesam:title")?,
            artists: get_string_vec(&map, "xesam:artist")?,
            album: get_string(&map, "xesam:album")?,
            artwork: get_string(&map, "mpris:artUrl")?,
        })
    }
}

fn get_string(map: &HashMap<String, OwnedValue>, key: &str) -> Result<String, MetadataError> {
    map.get(key)
        .and_then(|v| v.downcast_ref::<str>().map(|s| s.to_string()))
        .ok_or_else(|| MetadataError::MissingKey(key.to_string()))
}

fn get_string_vec(
    map: &HashMap<String, OwnedValue>,
    key: &str,
) -> Result<Vec<String>, MetadataError> {
    map.get(key)
        .and_then(|v| v.downcast_ref::<Array>())
        .map(|array| {
            array
                .get()
                .iter()
                .filter_map(|v| v.downcast_ref::<str>().map(|s| s.to_string()))
                .collect()
        })
        .ok_or_else(|| MetadataError::MissingKey(key.to_string()))
}

#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
enum Commands {
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
enum PlayMode {
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
struct Args {
    /// Changes the service that the DBus commands are sent to
    #[clap(
        short,
        long,
        value_parser,
        default_value = "org.mpris.MediaPlayer2.spotify"
    )]
    service_name: String,

    #[clap(subcommand)]
    action: Commands,
}

#[derive(Serialize, Deserialize, Debug)]
struct Response {
    tracks: Tracks,
}

#[derive(Serialize, Deserialize, Debug)]
struct Tracks {
    items: Vec<Track>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Track {
    name: String,
    id: String,
    artists: Vec<Artist>,
    album: Album,
}

impl Display for Track {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let artists = self
            .artists
            .iter()
            .map(|a| a.name.clone())
            .collect::<Vec<_>>();
        let (last, start) = artists.split_last().unwrap();
        let artists = start.join(", ");
        let artist = if artists.is_empty() {
            last.to_string()
        } else {
            format!("{} and {}", artists, last)
        };
        write!(f, "{} by {} on {}", self.name, artist, self.album.name)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Artist {
    name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Album {
    name: String,
}

async fn search(query: &str) -> Result<Vec<Track>, Error> {
    let url = format!(
        "https://spotify-search-api-test.herokuapp.com/search/tracks?track={}",
        query.replace(' ', "%20")
    );
    let res: Response = reqwest::get(&url).await?.json().await?;
    Ok(res.tracks.items)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = Args::parse();

    let conn = zbus::Connection::session().await?;

    let proxy = PlayerProxy::builder(&conn)
        .destination(BusName::try_from(args.service_name.clone())?)?
        .build()
        .await?;

    match args.action {
        Commands::Next => proxy.next().await?,
        Commands::Previous => proxy.previous().await?,
        Commands::PlayPause => proxy.play_pause().await?,
        Commands::NowPlaying => {
            let metadata = proxy.metadata().await?;
            what(metadata.try_into()?).await?;
        }
        Commands::PlaySong { mode } => play_song(&proxy, mode).await?,
    }

    Ok(())
}

async fn what(metadata: Metadata) -> Result<(), Error> {
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

async fn play_song<'proxy>(proxy: &PlayerProxy<'proxy>, mode: PlayMode) -> Result<(), Error> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::mock;
    use mockall::predicate::*;

    mock! {
        Player {}
        #[async_trait]
        trait Player {
            async fn play_pause(&self) -> zbus::Result<()>;
            async fn next(&self) -> zbus::Result<()>;
            async fn previous(&self) -> zbus::Result<()>;
            async fn open_uri(&self, uri: &str) -> zbus::Result<()>;
            async fn metadata(&self) -> zbus::Result<HashMap<String, OwnedValue>>;
        }
    }

    #[tokio::test]
    async fn test_play_pause() {
        let mut mock = MockPlayer::new();
        mock.expect_play_pause().times(1).returning(|| Ok(()));

        let proxy = mock;
        proxy.play_pause().await.unwrap();
    }

    #[tokio::test]
    async fn test_next() {
        let mut mock = MockPlayer::new();
        mock.expect_next().times(1).returning(|| Ok(()));

        let proxy = mock;
        proxy.next().await.unwrap();
    }

    #[tokio::test]
    async fn test_previous() {
        let mut mock = MockPlayer::new();
        mock.expect_previous().times(1).returning(|| Ok(()));

        let proxy = mock;
        proxy.previous().await.unwrap();
    }

    #[tokio::test]
    async fn test_open_uri() {
        let mut mock = MockPlayer::new();
        mock.expect_open_uri()
            .with(eq("spotify:track:1234567890"))
            .times(1)
            .returning(|_| Ok(()));

        let proxy = mock;
        proxy.open_uri("spotify:track:1234567890").await.unwrap();
    }

    #[test]
    fn test_metadata_conversion() {
        let mut map = HashMap::new();
        map.insert("xesam:title".to_string(), OwnedValue::from("Test Title"));
        map.insert(
            "xesam:artist".to_string(),
            OwnedValue::from(vec!["Test Artist".to_string()]),
        );
        map.insert("xesam:album".to_string(), OwnedValue::from("Test Album"));
        map.insert(
            "mpris:artUrl".to_string(),
            OwnedValue::from("http://example.com/art.jpg"),
        );

        let metadata: Metadata = map.try_into().unwrap();
        assert_eq!(metadata.title, "Test Title");
        assert_eq!(metadata.artists, vec!["Test Artist"]);
        assert_eq!(metadata.album, "Test Album");
        assert_eq!(metadata.artwork, "http://example.com/art.jpg");
    }

    #[test]
    fn test_metadata_conversion_missing_key() {
        let mut map = HashMap::new();
        map.insert("xesam:title".to_string(), OwnedValue::from("Test Title"));
        // Missing "xesam:artist" key
        map.insert("xesam:album".to_string(), OwnedValue::from("Test Album"));
        map.insert(
            "mpris:artUrl".to_string(),
            OwnedValue::from("http://example.com/art.jpg"),
        );

        let result: Result<Metadata, MetadataError> = map.try_into();
        assert!(matches!(result, Err(MetadataError::MissingKey(key)) if key == "xesam:artist"));
    }
}
