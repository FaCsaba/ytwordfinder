use std::{
    ffi::OsStr,
    fs::{self, read_to_string, remove_file, File},
    io::{BufRead, BufReader, Write},
    process::Command,
};

use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};

lazy_static! {
    static ref TIME_STAMP_RE: Regex =
        Regex::new(r#"(\d{2,}):(\d{2}):(\d{2})[,|.]\d{3} --> (\d{2,}):(\d{2}):(\d{2})[,|.]\d{3}"#)
            .unwrap();
}

pub fn download_subtitle(link: String, lang: Option<String>) -> Result<(), String> {
    let c = Command::new("yt-dlp")
        .args([
            "--sub-langs",
            &lang.unwrap_or("zh-CN,en".to_string()),
            "--write-subs",
            &link,
            "--no-playlist",
            "-o",
            "subtitles/%(webpage_url_domain)s:%(id)s",
            "--skip-download",
        ])
        .output();

    downloaded_subs_to_subtitle_json();
    match c {
        Ok(_) => Ok(()),
        Err(_) => Err("Failed to execute yt-dlp. Is it downloaded?".to_string()),
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum VideoSite {
    Youtube,
    BiliBili,
    Unknown,
}

impl VideoSite {
    pub fn to_url(&self) -> String {
        match self {
            VideoSite::Youtube => "https://youtube.com/embed/".to_owned(),
            VideoSite::BiliBili => "https://player.bilibili.com/player.html?bvid=".to_owned(),
            VideoSite::Unknown => "".to_owned(),
        }
    }
}

impl From<&'_ str> for VideoSite {
    fn from(s: &'_ str) -> Self {
        match s {
            "youtube.com" => VideoSite::Youtube,
            "bilibili.com" => VideoSite::BiliBili,
            _ => VideoSite::Unknown,
        }
    }
}

impl Default for VideoSite {
    fn default() -> Self {
        Self::Unknown
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Lang {
    En,
    Chinese,
    Unknown,
}

impl From<String> for Lang {
    fn from(s: String) -> Self {
        match s.as_str() {
            "en" | "en-GB" | "en-US" => Lang::En,
            "zh-CN" | "zh-Hans" => Lang::Chinese,
            _ => Lang::Unknown,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct VideoTimeSubtitle {
    content: String,
    link: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct SubtitleContent {
    start_time: u32,
    end_time: u32,
    content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Subtitle {
    site: VideoSite,
    video_id: String,
    lang: Lang,
    content: Vec<SubtitleContent>,
}

impl Subtitle {
    pub fn push_subtitle_content(&mut self, content: String, time_stamp: String) {
        let mut times = TIME_STAMP_RE
            .captures(&time_stamp)
            .unwrap()
            .iter()
            .enumerate()
            .filter(|(i, _)| i != &0)
            .map(|(_, m)| m)
            .take_while(|x| x.is_some())
            .map(|t| t.unwrap().as_str().parse::<u32>().unwrap())
            .collect::<Vec<_>>();
        times[0] *= 60 * 60;
        times[1] *= 60;
        let start_time_slice = &times[0..3];
        let start_time: u32 = start_time_slice.iter().fold(0, |acc, x| acc + x);
        times[3] *= 60 * 60;
        times[4] *= 60;
        let end_time_slice = &times[3..6];
        let end_time = end_time_slice.iter().fold(0, |acc, x| acc + x);
        self.content.push(SubtitleContent {
            content,
            start_time,
            end_time,
        });
    }

    fn to_link(&self, sub: &SubtitleContent) -> String {
        match self.site {
            VideoSite::Youtube => format!(
                "{}{}?start={}",
                self.site.to_url(),
                self.video_id,
                if sub.start_time != 0 {
                    sub.start_time - 1
                } else {
                    0
                }
            ),
            VideoSite::BiliBili => format!(
                "{}{}&page=1&high_quality=1&danmaku=0&as_wide=1&t={}",
                self.site.to_url(),
                self.video_id,
                if sub.start_time != 0 {
                    sub.start_time - 1
                } else {
                    0
                }
            ),
            VideoSite::Unknown => todo!(),
        }
    }

    pub fn find_subtitle(&self, word: &String) -> Vec<VideoTimeSubtitle> {
        self.content
            .iter()
            .filter(|c| c.content.contains(word))
            .map(|sub| VideoTimeSubtitle {
                content: sub.content.clone(),
                link: self.to_link(sub),
            })
            .collect()
    }
}

fn downloaded_subs_to_subtitle_json() {
    let subtitles_dir = fs::read_dir("./subtitles");
    if subtitles_dir.is_err() {
        return;
    };
    let sub_files = subtitles_dir.unwrap();
    sub_files.for_each(|sub_file| {
        let sub_dir_entry = sub_file.unwrap();
        let file_name = sub_dir_entry.file_name();
        let sub_file_name = file_name.to_str().unwrap().split(":").collect::<Vec<_>>();
        let site = VideoSite::from(sub_file_name[0]);
        let video_id = sub_file_name[1].split(".").collect::<Vec<_>>()[0].to_string();
        let lang = Lang::from(sub_file_name[1].split(".").collect::<Vec<_>>()[1].to_string());

        let reader = BufReader::new(File::open(sub_dir_entry.path()).unwrap());
        let lines = reader.lines().map(|l| l.unwrap()).enumerate();

        let mut subtitle = Subtitle {
            video_id,
            site,
            lang,
            content: vec![],
        };
        match sub_dir_entry.path().extension().unwrap().to_str().unwrap() {
            "srt" => {
                let json_name = file_name.to_str().unwrap().replace("srt", "json");
                let mut f = File::create(format!("./subtitles/{}", json_name)).unwrap();
                let mut time_stamp: Option<String> = None;
                for line in lines {
                    if TIME_STAMP_RE.is_match(&line.1) {
                        time_stamp = Some(line.1.clone());
                        continue;
                    }
                    if line.1 == "" {
                        time_stamp = None
                    }
                    if time_stamp.is_some() {
                        subtitle.push_subtitle_content(line.1, time_stamp.clone().unwrap())
                    }
                }

                f.write_all(serde_json::to_string(&subtitle).unwrap().as_bytes())
                    .unwrap();
                remove_file(sub_dir_entry.path()).unwrap();
            }
            "vtt" => {
                let json_name = file_name.to_str().unwrap().replace("vtt", "json");
                let mut f = File::create(format!("./subtitles/{}", json_name)).unwrap();

                let mut time_stamp: Option<String> = None;
                for line in lines {
                    if TIME_STAMP_RE.is_match(&line.1) {
                        time_stamp = Some(line.1.clone());
                        continue;
                    }
                    if line.1 == "" {
                        time_stamp = None
                    }
                    if time_stamp.is_some() {
                        subtitle.push_subtitle_content(line.1, time_stamp.clone().unwrap())
                    }
                }

                f.write_all(serde_json::to_string(&subtitle).unwrap().as_bytes())
                    .unwrap();
                remove_file(sub_dir_entry.path()).unwrap();
            }
            &_ => {}
        }
    });
}

pub fn search_for_word(word: String) -> Vec<VideoTimeSubtitle> {
    let mut subtitles_dir = fs::read_dir("./subtitles");
    if subtitles_dir.is_err() {
        fs::create_dir("./subtitles").unwrap();
        subtitles_dir = fs::read_dir("./subtitles");
    }
    let paths = subtitles_dir.unwrap();
    paths
        .filter(|path| path.as_ref().unwrap().path().extension() == Some(&OsStr::new("json")))
        .map(|path| {
            let sub: Subtitle =
                serde_json::from_str(&read_to_string(path.unwrap().path()).unwrap()).unwrap();
            sub.find_subtitle(&word)
        })
        .flatten()
        .collect::<Vec<VideoTimeSubtitle>>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_download_and_search() -> Result<(), String> {
        download_subtitle("Jcq6tCLm8Bg".to_owned(), Some("zh-CN".to_owned()))?;
        let res = search_for_word("妈".to_owned());
        dbg!(&res);
        assert!(res[0].content.contains("妈"));
        Ok(())
    }
}
