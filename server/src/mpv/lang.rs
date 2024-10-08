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

    pub const fn empty() -> Self {
        Self {
            title: None,
            lang: None,
        }
    }

    fn lang(&self) -> Matcher<'_> {
        Matcher::new(self.lang.as_deref())
    }

    fn title(&self) -> Matcher<'_> {
        Matcher::new(self.title.as_deref())
    }

    fn ilang(&self) -> Matcher<'_> {
        self.lang().case_insensitive()
    }

    fn ititle(&self) -> Matcher<'_> {
        self.title().case_insensitive()
    }
}

struct Matcher<'a> {
    inner: &'a str,
    str_cmp: fn(&str, &str) -> bool,
}

impl<'a> Matcher<'a> {
    fn new(inner: Option<&'a str>) -> Self {
        assert!(inner != Some(""));
        Self {
            inner: inner.unwrap_or(""),
            str_cmp: str::eq,
        }
    }

    fn case_insensitive(mut self) -> Self {
        self.str_cmp = str::eq_ignore_ascii_case;
        self
    }

    fn equals(&self, b: &str) -> bool {
        (self.str_cmp)(self.inner, b)
    }

    fn contains(&self, word: &str) -> bool {
        words(self.inner).any(|w| (self.str_cmp)(w, word))
    }

    fn any_equals(&self, words: impl IntoIterator<Item = impl AsRef<str>>) -> bool {
        words.into_iter().any(|w| self.equals(w.as_ref()))
    }

    fn any_contains(&self, words: impl IntoIterator<Item = impl AsRef<str>>) -> bool {
        words.into_iter().any(|w| self.contains(w.as_ref()))
    }
}

fn words(sentence: &str) -> impl Iterator<Item = &str> {
    sentence
        .split(|c: char| !c.is_alphabetic())
        .filter(|w| !w.is_empty())
}

impl fmt::Display for Lang {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut written = false;

        if let Some(lang) = &self.lang {
            write!(f, "{lang}")?;
            written = true;
        }

        if let Some(title) = &self.title {
            if written {
                write!(f, " ")?;
            }
            write!(f, "'{title}'")?;
            written = true;
        }

        if !written {
            write!(f, "None")?;
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
        let selected = tracks
            .iter()
            .enumerate()
            .find(|(_, t)| t.selected)
            .map(|(i, _)| i)
            .unwrap_or(usize::MAX);
        log::info!("Available {type_name}: {:?} (selected={selected})", names);

        let chosen = auto_choose(tracks, selected, preferred);
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

// TODO: the english is very sub focused, and japanese contains multiple languages, so
// this is maybe not a good way to choose which language to use. Maybe introduce another
// enum for sub or dub?
#[derive(Copy, Clone, Debug)]
pub enum HumanLang {
    English,
    Japanese,
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Clone, Copy)]
enum Prio {
    /// Actively avoid using
    Avoid,
    /// Can use, but prefer not to
    NotUse,
    /// Perfectly fine to use
    Use,
    /// Prefer using this
    Prefer,
    /// Such a good match that it must be used
    Required,
}

impl HumanLang {
    fn choose(self, lang: &Lang) -> Prio {
        match self {
            Self::English => choose_eng_sub(lang),
            Self::Japanese => choose_jap_swe_eng_dub(lang),
        }
    }
}

fn auto_choose<It, T>(tracks: It, selected: usize, human: HumanLang) -> Option<(usize, T)>
where
    T: AsRef<Lang>,
    It: IntoIterator<Item = T>,
{
    let mut selected_prio = Prio::NotUse;
    let Some((max_i, max_t, max_prio)) = tracks
        .into_iter()
        .enumerate()
        .map(|(i, t)| {
            let prio = human.choose(t.as_ref());
            if i == selected {
                selected_prio = prio;
            }
            (i, t, prio)
        })
        .filter(|(_, _, prio)| *prio >= Prio::NotUse)
        .max_by(|(_, _, prio1), (_, _, prio2)| {
            prio1.cmp(prio2).then(std::cmp::Ordering::Greater)
        })
    else {
        return None;
    };

    if selected_prio == Prio::Avoid || max_prio >= Prio::Use {
        Some((max_i, max_t))
    } else {
        None
    }
}

fn choose_eng_sub(lang: &Lang) -> Prio {
    // NOTE: some youtube videos have a subtitle track called "live_chat 'json'" thats
    // empty
    let is_avoid = lang.ilang().equals("live_chat");
    if is_avoid {
        return Prio::Avoid;
    }

    let is_english = lang.ilang().any_equals(["eng", "en-US", "en", "english"]);
    if !is_english {
        return Prio::NotUse;
    }

    let is_specific = lang.ititle().any_equals(["for japanese audio", "dialogue"]);
    if is_specific {
        return Prio::Required;
    }

    let is_bad = lang
        .ititle()
        .any_contains(["signs", "forced", "dub", "dubtitle"])
        || lang.title().any_contains(["SDH"]);
    if is_bad {
        return Prio::Use;
    }

    Prio::Prefer
}

fn choose_jap_swe_eng_dub(lang: &Lang) -> Prio {
    if lang.ilang().any_equals(["ja", "jpn", "japanese"]) {
        return Prio::Required;
    }

    if lang.ilang().any_equals(["swe", "svenska", "sv"]) {
        return Prio::Prefer;
    }

    if lang.ilang().any_equals(["en", "english", "eng"]) {
        return Prio::Use;
    }

    Prio::NotUse
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn testing_words() {
        assert!(words("").next().is_none());
        assert_eq!(vec!("en", "SDH"), words("en (SDH)").collect::<Vec<_>>());
    }

    #[allow(dead_code)]
    impl Lang {
        // TODO: make these const and create shared global langs that all tests share
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
    fn empty_lang_is_notuse() {
        assert_eq!(Prio::NotUse, choose_jap_swe_eng_dub(&Lang::empty()));
        assert_eq!(Prio::NotUse, choose_eng_sub(&Lang::empty()));
    }

    #[test]
    fn empty_vec_does_nothing() {
        let chosen: Option<(_, Lang)> = auto_choose(vec![], 0, HumanLang::English);
        assert_eq!(None, chosen);
    }

    #[test]
    fn dont_set_to_empty_if_all_are_notuse() {
        let italian = Lang::new_lang("ita");
        assert_eq!(Prio::NotUse, choose_jap_swe_eng_dub(&italian));

        let chosen = auto_choose(vec![Lang::empty(), italian], 1, HumanLang::Japanese);
        assert_eq!(None, chosen);
    }

    #[test]
    fn avoiding_selected() {
        let avoid = Lang::new_both("json", "live_chat");
        assert_eq!(Prio::Avoid, choose_eng_sub(&avoid));

        let chosen =
            auto_choose(vec![Lang::empty(), avoid.clone()], 1, HumanLang::English)
                .map(|(i, _)| i);
        assert_eq!(Some(0), chosen);

        let chosen =
            auto_choose(vec![Lang::empty(), avoid.clone()], 0, HumanLang::English);
        assert_eq!(None, chosen);

        let chosen =
            auto_choose(vec![avoid.clone(), avoid.clone()], 1, HumanLang::English);
        assert_eq!(None, chosen);

        let chosen = auto_choose(vec![avoid], 0, HumanLang::English);
        assert_eq!(None, chosen);
    }

    #[test]
    // TODO: this probably fits better as a doc example, but that only works on lib
    // crates, not bin...
    // https://github.com/rust-lang/rust/issues/50784
    fn different_kinds_of_arguments() {
        let v: Vec<Lang> = Vec::new();
        auto_choose(v, 0, HumanLang::English);

        let v: Vec<Lang> = Vec::new();
        auto_choose(&v, 0, HumanLang::English);

        let v: Vec<&Lang> = Vec::new();
        auto_choose(v, 0, HumanLang::English);

        let v: Vec<&Lang> = Vec::new();
        auto_choose(&v, 0, HumanLang::English);

        assert!(true);
    }

    #[test]
    fn english_prio() {
        let preferred = vec![
            Lang::new_both("For Japanese audio", "eng"),
            Lang::new_lang("eng"),
            Lang::new_both("SDH", "en"),
            Lang::new_both("Forced", "en"),
            Lang::new_both("Signs", "eng"),
            Lang::new_lang("swe"),
            Lang::empty(),
            Lang::new_both("json", "live_chat"),
        ];
        let prios: Vec<_> = preferred.iter().map(choose_eng_sub).collect();
        assert!(prios.windows(2).all(|pair| pair[0] >= pair[1]));
    }

    #[test]
    fn jap_swe_en_prio() {
        let preferred = vec![
            Lang::new_lang("jpn"),
            Lang::new_lang("swe"),
            Lang::new_lang("eng"),
            Lang::empty(),
        ];
        let prios: Vec<_> = preferred.iter().map(choose_jap_swe_eng_dub).collect();
        assert!(prios.windows(2).all(|pair| pair[0] > pair[1]));
    }

    #[test]
    fn skips_signs() {
        let chosen = auto_choose(
            vec![
                Lang::new_both("Signs", "eng"),
                Lang::new_both("Dialogue", "eng"),
            ],
            0,
            HumanLang::English,
        );
        assert_eq!(Some(1), chosen.map(|(i, _)| i));
    }

    #[test]
    fn selects_selected_use() {
        let chosen = auto_choose(
            vec![Lang::empty(), Lang::new_lang("eng")],
            1,
            HumanLang::English,
        );
        assert_eq!(Some(1), chosen.map(|(i, _)| i));
    }

    #[test]
    fn sdh_is_case_sensitive() {
        assert_ne!(
            choose_eng_sub(&Lang::new_both("SDH", "eng")),
            choose_eng_sub(&Lang::new_both("sdh", "eng"))
        );
    }

    #[test]
    fn taking_the_leftmost_on_equal() {
        let chosen = auto_choose(
            vec![Lang::new_lang("eng"), Lang::new_lang("eng")],
            0,
            HumanLang::English,
        );
        assert_eq!(Some(0), chosen.map(|(i, _)| i));
    }

    #[test]
    fn very_descriptive_titles() {
        let chosen = auto_choose(
            vec![
                Lang::empty(),
                Lang::new_both("Forced (For English audio)", "en"),
                Lang::new_both("For Japanese audio", "en"),
            ],
            1,
            HumanLang::English,
        );
        assert_eq!(Some(2), chosen.map(|(i, _)| i));
    }

    #[test]
    fn for_japanese_audio() {
        let chosen = auto_choose(
            vec![
                Lang::empty(),
                Lang::new_both("For English audio", "en"),
                Lang::new_both("For Japanese audio", "en"),
            ],
            1,
            HumanLang::English,
        );
        assert_eq!(Some(2), chosen.map(|(i, _)| i));
    }

    #[test]
    fn no_dubtitles() {
        let chosen = auto_choose(
            vec![
                Lang::empty(),
                Lang::new_both("Dubtitle", "en"),
                Lang::new_lang("en"),
                Lang::new_both("Dub", "en"),
            ],
            1,
            HumanLang::English,
        );
        assert_eq!(Some(2), chosen.map(|(i, _)| i));
    }

    #[test]
    fn special_in_parens() {
        let chosen = auto_choose(
            vec![
                Lang::empty(),
                Lang::new_both("English (Forced)", "en"),
                Lang::new_lang("en"),
                Lang::new_both("English (SDH)", "en"),
            ],
            1,
            HumanLang::English,
        );
        assert_eq!(Some(2), chosen.map(|(i, _)| i));
    }

    #[test]
    fn deselects_italian_dubs() {
        let chosen = auto_choose(
            vec![
                Lang::empty(),
                Lang::new_both("AC3 5.1 ITA", "ita"),
                Lang::new_both("AC3 5.1 ENG", "eng"),
            ],
            1,
            HumanLang::Japanese,
        );
        assert_eq!(Some(2), chosen.map(|(i, _)| i));
    }

    #[test]
    fn selects_jap_before_eng() {
        let chosen = auto_choose(
            vec![Lang::empty(), Lang::new_lang("eng"), Lang::new_lang("jpn")],
            0,
            HumanLang::Japanese,
        );
        assert_eq!(Some(2), chosen.map(|(i, _)| i));
    }
}
