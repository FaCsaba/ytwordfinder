use std::{
    fs::{self, File},
    io::{BufRead, BufReader},
    process::Command,
};

use serde::Serialize;

pub fn download_subtitle(link: String, lang: Option<String>) -> Result<(), String> {
    let c = Command::new("youtube-dl")
        .args([
            "--sub-lang",
            &lang.unwrap_or("zh-CN".to_string()),
            "--write-sub",
            &link,
            "-o",
            "subtitles/%(id)s",
            "--skip-download",
        ])
        .output();

    match c {
        Ok(_) => Ok(()),
        Err(_) => Err("Failed to execute youtube-dl. Is it downloaded?".to_string()),
    }
}

#[derive(Debug, Serialize)]
pub struct VideoTime {
    #[serde(rename(serialize = "videoId"))]
    video_id: String,

    subtitle: String,

    #[serde(rename(serialize = "start"))]
    start_time_secs: u32,

    #[serde(rename(serialize = "end"))]
    end_time_secs: u32,
}

impl VideoTime {
    pub fn new(video_id: String, subtitle: String, time_stamp: String) -> Self {
        let mut time: Vec<u32> = time_stamp[0..8]
            .split(':')
            .map(|t| t.parse::<u32>().unwrap_or(0))
            .collect();
        time[0] = time[0] * 60 * 60;
        time[1] = time[1] * 60;
        let start_time_secs = time.iter().fold(0, |acc, s| acc + s);

        let mut time: Vec<u32> = time_stamp[17..25]
            .split(':')
            .map(|t| t.parse::<u32>().unwrap())
            .collect();
        time[0] = time[0] * 60 * 60;
        time[1] = time[1] * 60;
        let end_time_secs = time.iter().fold(0, |acc, s| acc + s);

        Self {
            video_id,
            subtitle,
            start_time_secs,
            end_time_secs,
        }
    }

    pub fn to_link(&self) -> String {
        format!(
            "https://youtube.com/embed/{}?t={}s",
            self.video_id, self.start_time_secs
        )
    }
}

pub fn search_for_word(word: String) -> Vec<VideoTime> {
    let paths = fs::read_dir("./subtitles").unwrap();
    paths
        .map(|path| {
            let reader = BufReader::new(File::open(path.as_ref().unwrap().path()).unwrap());
            let mut time_stamps: Vec<VideoTime> = vec![];
            let lines: Vec<String> = reader.lines().map(|l| l.unwrap()).collect();
            lines.iter().enumerate().for_each(|(i, r)| {
                if r.contains(&word) && lines[i - 1].len() > 8 {
                    time_stamps.push(VideoTime::new(
                        path.as_ref()
                            .unwrap()
                            .path()
                            .file_name()
                            .unwrap()
                            .to_str()
                            .unwrap()
                            .split(".")
                            .collect::<Vec<&str>>()[0]
                            .to_string(),
                        lines[i].clone(),
                        lines[i - 1].clone(),
                    ))
                }
            });
            time_stamps
        })
        .flatten()
        .collect::<Vec<VideoTime>>()
}
