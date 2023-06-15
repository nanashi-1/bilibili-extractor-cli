use std::{ffi::OsStr, fmt::Display, io::Error, path::Path};

use bilibili_extractor::{
    ass_to_srt::convert_ass_to_srt,
    ffmpeg_controller::merge,
    json_to_ass::convert_json_to_ass,
    metadata_reader::{get_season_metadata, EpisodeMetadata, SeasonMetadata},
    packager::{package_season, PackageConfig},
};
use clap::Parser;
use spinners::{Spinner, Spinners};

use crate::{error_handling::return_when_error, text_code::TextCode};

mod error_handling;
mod text_code;

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

    #[clap(short, long, help = "use srt subtitles")]
    srt: bool,

    #[clap(short, long, default_value_t=String::from("."), help="directory to store the encoded videos")]
    output: String,
}

pub fn list(download_path: &str) -> std::io::Result<()> {
    let download_directory = Path::new(&download_path)
        .read_dir()
        .unwrap_or_else(|_| panic!("{}", "Error reading download directory".as_error()));

    download_directory
        .map(|s| {
            let season_path = s?.path();

            let mut season_metadata = return_when_error(
                get_season_metadata(&season_path),
                &format!("Error reading season metadata at {:?}", season_path),
            )?;

            println!(
                "{}: {}\n{}:\n",
                "Season Title".as_primary_header(),
                season_metadata.title,
                "Episodes".as_primary_header()
            );

            season_metadata
                .episodes
                .sort_by(|a, b| format_index(&a.index).cmp(&format_index(&b.index)));

            season_metadata.episodes.iter().for_each(|e| {
                println!(
                    "{}: {}\n{}: {}\n{}: {:?}\n",
                    "Episode Title".as_secondary_header(),
                    e.title,
                    "Episode Index".as_secondary_header(),
                    e.index,
                    "Episode Path".as_secondary_header(),
                    e.path
                )
            });

            Ok(())
        })
        .collect::<Result<Vec<()>, Error>>()?;

    Ok(())
}

pub fn compile_seasons(
    download_path: &str,
    output_directory: &str,
    dont_copy: bool,
    hard_subtitle: bool,
    use_srt_subtitle: bool,
) -> std::io::Result<()> {
    let download_directory = Path::new(&download_path)
        .read_dir()
        .unwrap_or_else(|_| panic!("{}", "Error reading download directory".as_error()));

    download_directory
        .map(|s| {
            let season_path = s?.path();
            let season_metadata = return_when_error(
                get_season_metadata(&season_path),
                &format!("Error reading season metadata at {:?}", season_path),
            )?;

            season_metadata
                .episodes
                .iter()
                .map(|e| compile_episode(e, &season_metadata, hard_subtitle, use_srt_subtitle))
                .collect::<Result<Vec<()>, Error>>()?;

            return_when_error(
                package_season(
                    &season_path,
                    &Path::new(&output_directory).to_path_buf(),
                    PackageConfig {
                        copy: !dont_copy,
                        episode_video_path: &Path::new("episode.mkv").to_path_buf(),
                    },
                ),
                &format!(
                    "An error occured while pakaging season at {:?}",
                    season_path
                ),
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
    use_srt_subtitle: bool,
) -> std::io::Result<()> {
    let mut spinner = make_loading_message(format!(
        "Locating subtitle for: \"{}\"",
        episode_metadata.title
    ));

    let mut subtitle = episode_metadata
        .path
        .join("en")
        .read_dir()
        .unwrap_or_else(|_| panic!("Subtitle folder not found: {:?}", episode_metadata.path))
        .next()
        .expect("Subtitle is missing!")?
        .path();

    spinner.stop_and_persist("✔", "Finished locating subtitle.".as_primary_header());

    if subtitle.extension() == Some(OsStr::new("json")) {
        let mut spinner = make_loading_message("Translating JSON subtitle to ASS");

        let mut subtitle_out = subtitle.clone();
        subtitle_out.set_extension("ass");

        return_when_error(
            convert_json_to_ass(&subtitle, &subtitle_out),
            &format!(
                "Error occured while parsing JSON subtitle at {:?}",
                episode_metadata.path
            ),
        )?;

        subtitle.set_extension("ass");

        spinner.stop_and_persist(
            "✔",
            "Finished translating JSON subtitle to ASS.".as_primary_header(),
        );
    }

    if use_srt_subtitle {
        let mut spinner = make_loading_message("Converting ASS to SRT");

        let mut subtitle_out = subtitle.clone();
        subtitle_out.set_extension("srt");

        return_when_error(
            convert_ass_to_srt(
                subtitle.to_str().unwrap_or_default(),
                subtitle_out.to_str().unwrap_or_default(),
            ),
            &format!(
                "Error occured while parsing ASS subtitle at {:?}",
                episode_metadata.path
            ),
        )?;

        subtitle.set_extension("srt");

        spinner.stop_and_persist("✔", "Finished converting ASS to SRT.".as_primary_header());
    }

    let files_path = episode_metadata.path.join(&season_metadata.type_tag);

    let mut spinner = make_loading_message(format!(
        "Creating video for: \"{}\"",
        episode_metadata.title
    ));

    return_when_error(
        merge(
            files_path.join("video.m4s").to_str().unwrap(),
            files_path.join("audio.m4s").to_str().unwrap(),
            subtitle.to_str().unwrap(),
            episode_metadata.path.join("episode.mkv").to_str().unwrap(),
            hard_subtitle,
        ),
        &format!(
            "Error occured while merging files at {:?}",
            episode_metadata.path
        ),
    )?;

    spinner.stop_and_persist(
        "✔",
        format!("Creating video for: \"{}\"", episode_metadata.title).as_primary_header(),
    );

    Ok(())
}

fn make_loading_message(message: impl Into<String> + TextCode + Display) -> Spinner {
    Spinner::new(Spinners::Dots, message.as_primary_header())
}

fn main() {
    let arguments = Arguments::parse();

    if arguments.list {
        if let Err(e) = list(&arguments.path) {
            println!("{}", e.to_string().as_error());
            return;
        }
    }

    if let Err(e) = compile_seasons(
        &arguments.path,
        &arguments.output,
        arguments.dont_copy,
        arguments.hard_sub,
        arguments.srt,
    ) {
        println!("{}", e.to_string().as_error());
    }
}
