use std::{collections::HashMap, time::Duration};

use protocol::{
    to_client::front::Front,
    util::{Percent, PositivePercent},
    ToClientable,
};
use web_sys::window;

use crate::hooks::server::Accepted;

#[derive(Debug, PartialEq, Clone)]
pub struct Debug {
    pub path: Vec<String>,
    pub kv: HashMap<String, String>,
}

pub fn debug() -> Result<Debug, String> {
    if !cfg!(debug_assertions) {
        return Err("not compiled in debug mode".to_string());
    }

    let location = window()
        .ok_or_else(|| "failed to get window".to_string())?
        .location();

    let href = location
        .href()
        .map_err(|e| format!("failed to get href: {e:?}"))?;

    let url = url::Url::parse(&href).map_err(|e| format!("Failed to parse url: {e}"))?;

    let segments: Vec<&str> = url
        .path_segments()
        .ok_or_else(|| "url cannot-be-a-base".to_string())?
        .collect();

    let first = segments.first().expect("the vec is always non-empty");
    if *first != "debug" {
        return Err("not a valid debug URL".to_string());
    }

    let path: Vec<String> = segments.iter().skip(1).map(|s| s.to_string()).collect();
    let kv: HashMap<String, String> = url.query_pairs().into_owned().collect();

    Ok(Debug { path, kv })
}

impl Debug {
    pub fn is_connected(&self) -> bool {
        self.bool_kv("connected", true)
    }

    pub fn accepted(&self) -> Accepted {
        match self.kv.get("accepted").map(String::as_str) {
            Some("pending") => Accepted::Pending,
            Some("rejected") => Accepted::Rejected,
            _ => Accepted::Accepted,
        }
    }

    fn bool_kv(&self, key: &str, default: bool) -> bool {
        match self.kv.get(key).map(String::as_str) {
            Some("yes") | Some("true") | Some("") => true,
            Some("no") | Some("false") => false,
            _ => default,
        }
    }

    fn string_kv(&self, key: &str, default: &str) -> String {
        self.kv
            .get(key)
            .map(String::as_str)
            .unwrap_or(default)
            .to_string()
    }

    fn f64_kv(&self, key: &str, default: f64) -> f64 {
        let Some(s) = self.kv.get(key) else {
            return default;
        };

        let Ok(f) = s.parse() else {
            return default;
        };

        f
    }

    pub fn front(&self) -> Front {
        if self.path.is_empty() {
            return Front::None;
        }

        match self.path.first().unwrap().as_str() {
            "mpv" => self.front_mpv(),
            _ => Front::None,
        }
    }

    fn front_mpv(&self) -> Front {
        use protocol::to_client::front::mpv;
        use protocol::to_client::front::mpv::playstate;
        use protocol::to_client::front::mpv::playstate::Track;
        assert_eq!(Some("mpv"), self.path.first().map(String::as_str));

        let subpage = self.path.get(1).map(String::as_str);
        let toclient = match subpage {
            Some("play") => playstate::PlayState {
                title: self.string_kv("title", "Exempelvideofilm"),
                pause: self.bool_kv("pause", false),
                progress: Duration::from_secs(1111),
                length: Duration::from_secs(3651),
                volume: (!self.bool_kv("muted", false)).then(|| {
                    PositivePercent::try_new(self.f64_kv("volume", 80.0))
                        .unwrap_or_default()
                }),
                chapter: None,
                subtitles: vec![Track {
                    id: 0,
                    title: "None".to_string(),
                    selected: true,
                }],
                audios: vec![Track {
                    id: 0,
                    title: "None".to_string(),
                    selected: true,
                }],
            }
            .to_client(),
            _ => mpv::Load.to_client(),
        };

        match toclient {
            protocol::ToClient::Front(front) => front,
            protocol::ToClient::Seat(_) => unreachable!("will always be a front"),
        }
    }
}
