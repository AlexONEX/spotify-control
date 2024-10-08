mod commands;
mod error;
mod metadata;
mod notification;
mod player;
mod spotify_api;

use clap::Parser;
use commands::{Args, Commands, PlayMode};
use player::PlayerProxy;
use std::io::Write;
use zbus::names::BusName;

#[tokio::main]
async fn main() -> Result<(), error::Error> {
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
            notification::what(metadata.try_into()?).await?;
        }
        Commands::PlaySong { mode } => play_song(&proxy, mode).await?,
    }
    Ok(())
}

async fn play_song<'a>(proxy: &'a PlayerProxy<'a>, mode: PlayMode) -> Result<(), error::Error> {
    match mode {
        PlayMode::Uri { uri } => {
            proxy.open_uri(&uri).await?;
            Ok(())
        }
        PlayMode::Search { query, list, count } => {
            let query = query.join(" ");
            let tracks = spotify_api::search(&query).await?;
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
