use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Serialize, Deserialize, Debug)]
struct Response {
    tracks: Tracks,
}

#[derive(Serialize, Deserialize, Debug)]
struct Tracks {
    items: Vec<Track>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Track {
    pub name: String,
    pub id: String,
    pub artists: Vec<Artist>,
    pub album: Album,
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
pub struct Artist {
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Album {
    pub name: String,
}

pub async fn search(query: &str) -> Result<Vec<Track>, crate::error::Error> {
    let url = format!(
        "https://spotify-search-api-test.herokuapp.com/search/tracks?track={}",
        query.replace(' ', "%20")
    );
    let res: Response = reqwest::get(&url).await?.json().await?;
    Ok(res.tracks.items)
}
