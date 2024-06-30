use std::fmt;

use anyhow::Context;

use super::{MpvResult, Track, TrackType, DEF_USR};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lang {
    title: Option<String>,
    lang: Option<String>,
}

impl AsRef<Lang> for Lang {
    fn as_ref(&self) -> &Lang {
        self
    }
}

impl Lang {
    pub fn new(title: Option<String>, lang: Option<String>) -> Self {
        assert!(title.as_deref() != Some(""));
        assert!(lang.as_deref() != Some(""));
        Self { title, lang }
    }

    fn lang(&self) -> Matcher<'_> {
        Matcher {
            inner: self.lang.as_deref(),
        }
    }

    fn title(&self) -> Matcher<'_> {
        Matcher {
            inner: self.title.as_deref(),
        }
    }
}

struct Matcher<'a> {
    inner: Option<&'a str>,
}

impl<'a> Matcher<'a> {
    fn iequals(&self, b: &str) -> bool {
        match self.inner {
            None => false,
            Some(a) => a.eq_ignore_ascii_case(b),
        }
    }
}

impl fmt::Display for Lang {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(lang) = &self.lang {
            write!(f, "{lang}")?;
            if self.title.is_some() {
                write!(f, " ")?;
            }
        }

        if let Some(title) = &self.title {
            write!(f, "'{title}'")?;
        }

        Ok(())
    }
}

pub struct AutoLang {
    has_chosen: bool,
    preferred_sub: HumanLang,
    preferred_audio: HumanLang,
}

impl AutoLang {
    pub fn new(sub: HumanLang, dub: HumanLang) -> Self {
        Self {
            has_chosen: false,
            preferred_sub: sub,
            preferred_audio: dub,
        }
    }

    pub fn auto_choose(
        &mut self,
        mpv: &mut libmpv::Handle<libmpv::Async>,
        tracks: &[Track],
    ) -> MpvResult<()> {
        self.has_chosen = true;

        log::info!("Performing automatic track selection");

        if let Some(id) = Self::choose_track(tracks, TrackType::Sub, self.preferred_sub) {
            mpv.set_sub(id)
                .asynch(DEF_USR)
                .context("auto setting the sub")?;
        }

        if let Some(id) =
            Self::choose_track(tracks, TrackType::Audio, self.preferred_audio)
        {
            mpv.set_audio(id)
                .asynch(DEF_USR)
                .context("auto setting the audio")?;
        }

        Ok(())
    }

    fn choose_track(
        tracks: &[Track],
        ttype: TrackType,
        preferred: HumanLang,
    ) -> Option<i64> {
        let tracks: Vec<_> = tracks.iter().filter(|t| t.ttype == ttype).collect();

        let names: Vec<_> = tracks.iter().map(|track| track.lang.to_string()).collect();
        let type_name = match ttype {
            TrackType::Audio => "dubs",
            TrackType::Video => "vubs",
            TrackType::Sub => "subs",
        };
        log::info!("Available {type_name}: {:?}", names);

        let chosen = auto_choose(tracks, preferred);
        match chosen {
            Some((i, track)) => log::info!("Chose: {} ({i})", track.lang),
            None => log::info!("Chose nothing"),
        }

        chosen.map(|(_, track)| track.id)
    }

    pub fn has_not_chosen(&self) -> bool {
        !self.has_chosen
    }
}

#[derive(Copy, Clone, Debug)]
pub enum HumanLang {
    English,
    Japanese,
}

type Prio = u8;

impl HumanLang {
    fn choose(self, lang: &Lang) -> Prio {
        match self {
            Self::English => choose_eng(lang),
            Self::Japanese => choose_jap(lang),
        }
    }
}

fn auto_choose<It, T>(tracks: It, human: HumanLang) -> Option<(usize, T)>
where
    T: AsRef<Lang>,
    It: IntoIterator<Item = T>,
{
    tracks
        .into_iter()
        .enumerate()
        .map(|(i, t)| {
            let prio = human.choose(t.as_ref());
            (i, t, prio)
        })
        .filter(|(_, _, prio)| *prio > 0)
        .max_by(|(_, _, prio1), (_, _, prio2)| {
            prio1.cmp(prio2).then(std::cmp::Ordering::Greater)
        })
        .map(|(i, t, _)| (i, t))
}

fn choose_eng(lang: &Lang) -> Prio {
    let is_english = ["eng", "en-US", "en", "english"]
        .into_iter()
        .any(|s| lang.lang().iequals(s));

    let is_special = ["SDH", "signs"]
        .into_iter()
        .any(|s| lang.title().iequals(s));

    is_english.then_some(1).unwrap_or(0) + is_special.then_some(0).unwrap_or(1)
}

fn choose_jap(lang: &Lang) -> Prio {
    (lang.lang().iequals("ja") || lang.lang().iequals("jpn"))
        .then_some(1)
        .unwrap_or(0)
}

#[cfg(test)]
mod test {
    use super::*;

    #[allow(dead_code)]
    impl Lang {
        fn new_title(title: impl Into<String>) -> Self {
            Self::new(Some(title.into()), None)
        }

        fn new_lang(lang: impl Into<String>) -> Self {
            Self::new(None, Some(lang.into()))
        }

        fn new_both(title: impl Into<String>, lang: impl Into<String>) -> Self {
            Self::new(Some(title.into()), Some(lang.into()))
        }
    }

    #[test]
    // TODO: this probably fits better as a doc example
    fn different_kinds_of_arguments() {
        let v: Vec<Lang> = Vec::new();
        auto_choose(v, HumanLang::English);

        let v: Vec<Lang> = Vec::new();
        auto_choose(&v, HumanLang::English);

        let v: Vec<&Lang> = Vec::new();
        auto_choose(v, HumanLang::English);

        let v: Vec<&Lang> = Vec::new();
        auto_choose(&v, HumanLang::English);

        assert!(true);
    }

    #[test]
    fn english_prio() {
        let preferred = vec![
            Lang::new_lang("eng"),
            Lang::new_both("SDH", "en"),
            Lang::new_both("Signs", "eng"),
            Lang::new_lang("swe"),
        ];
        let prios: Vec<_> = preferred.iter().map(choose_eng).collect();
        assert!(prios.windows(2).all(|pair| pair[0] >= pair[1]));
    }

    #[test]
    fn skips_signs() {
        let chosen = auto_choose(
            vec![
                Lang::new_both("Signs", "eng"),
                Lang::new_both("Dialogue", "eng"),
            ],
            HumanLang::English,
        );
        assert_eq!(Some(1), chosen.map(|(i, _)| i));
    }

    #[test]
    fn taking_the_leftmost_on_equal() {
        let chosen = auto_choose(
            vec![Lang::new_lang("eng"), Lang::new_lang("eng")],
            HumanLang::English,
        );
        assert_eq!(Some(0), chosen.map(|(i, _)| i));
    }
}
