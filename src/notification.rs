use crate::error::Error;
use crate::metadata::Metadata;
use notify_rust::{Hint, Notification};

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
