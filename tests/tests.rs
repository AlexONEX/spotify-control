use clap::Parser;
use spotify_control::commands::{Args, Commands, PlayMode};

#[test]
fn test_next_command() {
    let args = Args::try_parse_from(&["spotify-control", "next"]).unwrap();
    assert_eq!(args.action, Commands::Next);
}

#[test]
fn test_previous_command() {
    let args = Args::try_parse_from(&["spotify-control", "previous"]).unwrap();
    assert_eq!(args.action, Commands::Previous);
}

#[test]
fn test_playpause_command() {
    let args = Args::try_parse_from(&["spotify-control", "play-pause"]).unwrap();
    assert_eq!(args.action, Commands::PlayPause);
}

#[test]
fn test_nowplaying_command() {
    let args = Args::try_parse_from(&["spotify-control", "now-playing"]).unwrap();
    assert_eq!(args.action, Commands::NowPlaying);
}

#[test]
fn test_playsong_uri_command() {
    let args = Args::try_parse_from(&[
        "spotify-control",
        "play-song",
        "uri",
        "spotify:track:1234567890",
    ])
    .unwrap();
    assert_eq!(
        args.action,
        Commands::PlaySong {
            mode: PlayMode::Uri {
                uri: "spotify:track:1234567890".to_string()
            }
        }
    );
}

#[test]
fn test_playsong_search_command() {
    let args = Args::try_parse_from(&[
        "spotify-control",
        "play-song",
        "search",
        "Shape",
        "of",
        "You",
    ])
    .unwrap();
    assert_eq!(
        args.action,
        Commands::PlaySong {
            mode: PlayMode::Search {
                query: vec!["Shape".to_string(), "of".to_string(), "You".to_string()],
                list: false,
                count: 5
            }
        }
    );
}

#[test]
fn test_playsong_search_with_list_command() {
    let args = Args::try_parse_from(&[
        "spotify-control",
        "play-song",
        "search",
        "--list",
        "Shape",
        "of",
        "You",
    ])
    .unwrap();
    assert_eq!(
        args.action,
        Commands::PlaySong {
            mode: PlayMode::Search {
                query: vec!["Shape".to_string(), "of".to_string(), "You".to_string()],
                list: true,
                count: 5
            }
        }
    );
}

#[test]
fn test_playsong_search_with_count_command() {
    let args = Args::try_parse_from(&[
        "spotify-control",
        "play-song",
        "search",
        "--count",
        "10",
        "Shape",
        "of",
        "You",
    ])
    .unwrap();
    assert_eq!(
        args.action,
        Commands::PlaySong {
            mode: PlayMode::Search {
                query: vec!["Shape".to_string(), "of".to_string(), "You".to_string()],
                list: false,
                count: 10
            }
        }
    );
}

#[test]
fn test_custom_service_name() {
    let args = Args::try_parse_from(&[
        "spotify-control",
        "--service-name",
        "org.mpris.MediaPlayer2.custom",
        "next",
    ])
    .unwrap();
    assert_eq!(args.service_name, "org.mpris.MediaPlayer2.custom");
    assert_eq!(args.action, Commands::Next);
}
