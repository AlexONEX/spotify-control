use std::collections::HashMap;
use zbus::dbus_proxy;
use zbus::zvariant::OwnedValue;

#[dbus_proxy(
    interface = "org.mpris.MediaPlayer2.Player",
    default_path = "/org/mpris/MediaPlayer2"
)]
pub trait Player {
    fn play_pause(&self) -> zbus::Result<()>;
    fn next(&self) -> zbus::Result<()>;
    fn previous(&self) -> zbus::Result<()>;
    fn open_uri(&self, uri: &str) -> zbus::Result<()>;
    #[dbus_proxy(property)]
    fn metadata(&self) -> zbus::Result<HashMap<String, OwnedValue>>;
}
