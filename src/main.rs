use clap::Parser;
use error::Error;
use player::{Commands, Player};

mod error;
mod metadata;
mod player;
mod spotify_api;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = player::Args::parse();

    let conn = zbus::Connection::session().await?;

    let proxy = player::PlayerProxy::builder(&conn)
        .destination(zbus::names::BusName::try_from(args.service_name.clone())?)?
        .build()
        .await?;

    match args.action {
        Commands::Next => proxy.next().await?,
        Commands::Previous => proxy.previous().await?,
        Commands::PlayPause => proxy.play_pause().await?,
        Commands::NowPlaying => {
            let metadata = proxy.metadata().await?;
            player::what(metadata.try_into()?).await?;
        }
        Commands::PlaySong { mode } => player::play_song(&proxy, mode).await?,
    }

    Ok(())
}
