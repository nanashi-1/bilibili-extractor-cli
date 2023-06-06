use std::{ffi::OsStr, io::Error, path::Path};

use bilibili_extractor::{
    ffmpeg_controller::merge,
    json_to_ass::convert_json_to_ass,
    metadata_reader::{get_season_metadata, EpisodeMetadata, SeasonMetadata},
    packager::{package_season, PackageConfig},
};
use clap::Parser;
use colored::Colorize;

#[derive(Parser)]
struct Arguments {
    #[clap(short, long, help = "list all the season in the download folder")]
    list: bool,

    #[clap(short, long, help = "move the episode.mkv instead of copying")]
    dont_copy: bool,

    #[clap(short, long, default_value_t = String::from("."), help="path to the download folder")]
    path: String,

    #[clap(
        short = 'H',
        long,
        help = "burn subtitle to the video as a hard subtitle"
    )]
    hard_sub: bool,

    #[clap(short, long, default_value_t=String::from("."), help="directory to store the encoded videos")]
    output: String,
}

pub fn list(download_path: String) -> std::io::Result<()> {
    let download_directory = Path::new(&download_path).read_dir()?;

    download_directory
        .map(|s| {
            let mut season_metadata = get_season_metadata(s?.path())?;

            println!(
                "{}: {}\n{}:\n",
                "Season Title".green().bold(),
                season_metadata.title,
                "Episodes".green().bold()
            );

            season_metadata
                .episodes
                .sort_by(|a, b| format_index(&a.index).cmp(&format_index(&b.index)));

            season_metadata.episodes.iter().for_each(|e| {
                println!(
                    "{}: {}\n{}: {}\n{}: {:?}\n",
                    "Episode Title".blue().bold(),
                    e.title,
                    "Episode Index".blue().bold(),
                    e.index,
                    "Episode Path".blue().bold(),
                    e.path
                )
            });

            Ok(())
        })
        .collect::<Result<Vec<()>, Error>>()?;

    Ok(())
}

pub fn compile_seasons(
    download_path: String,
    output_directory: String,
    dont_copy: bool,
    hard_subtitle: bool,
) -> std::io::Result<()> {
    let download_directory = Path::new(&download_path).read_dir()?;

    download_directory
        .map(|s| {
            let s = s?.path();
            let season_metadata = get_season_metadata(&s)?;

            season_metadata
                .episodes
                .iter()
                .map(|e| compile_episode(e, &season_metadata, hard_subtitle))
                .collect::<Result<Vec<()>, Error>>()?;

            package_season(
                s,
                Path::new(&output_directory).into(),
                PackageConfig {
                    copy: !dont_copy,
                    episode_video_path: Path::new("episode.mkv").into(),
                },
            )?;

            Ok(())
        })
        .collect::<Result<Vec<()>, Error>>()?;

    Ok(())
}

pub fn format_index(index: &str) -> String {
    format!("{index:0>2}")
}

fn compile_episode(
    episode_metadata: &EpisodeMetadata,
    season_metadata: &SeasonMetadata,
    hard_subtitle: bool,
) -> std::io::Result<()> {
    let mut subtitle = episode_metadata
        .path
        .join("en")
        .read_dir()
        .unwrap_or_else(|_| panic!("Subtitle folder not found: {:?}", episode_metadata.path))
        .next()
        .expect("Subtitle is missing!")?
        .path();

    if subtitle.extension() == Some(OsStr::new("json")) {
        let mut subtitle_out = subtitle.clone();
        subtitle_out.set_extension(".ass");

        convert_json_to_ass(&subtitle, &subtitle_out)?;

        subtitle.set_extension(".ass");
    }

    let files_path = episode_metadata.path.join(&season_metadata.type_tag);

    merge(
        files_path.join("video.m4s").to_str().unwrap(),
        files_path.join("audio.m4s").to_str().unwrap(),
        subtitle.to_str().unwrap(),
        episode_metadata.path.join("episode.mkv").to_str().unwrap(),
        hard_subtitle,
    )?;

    Ok(())
}

fn main() -> std::io::Result<()> {
    let arguments = Arguments::parse();

    if arguments.list {
        list(arguments.path)?;
        return Ok(());
    }

    compile_seasons(
        arguments.path,
        arguments.output,
        arguments.dont_copy,
        arguments.hard_sub,
    )?;

    Ok(())
}
