use serde::{
    de::{self, Deserialize as DeserializeTrait, Deserializer, Visitor},
    Deserialize,
};
use serde_json::Result;
use std::{
    cell::Cell, fmt, fs::File, ops::Deref, path::PathBuf, result::Result as StdResult,
    time::Duration,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Time(Duration);

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct Lyrics {
    pub error: bool,
    pub syncType: String,
    pub lines: Vec<LyricsEntry>,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(non_snake_case)]
pub struct LyricsEntry {
    pub startTimeMs: Cell<Time>,
    pub words: String,
    pub endTimeMs: Cell<Time>,
}

impl<'a> DeserializeTrait<'a> for Time {
    fn deserialize<D>(deserializer: D) -> StdResult<Self, D::Error>
    where
        D: Deserializer<'a>,
    {
        struct TimeVisitor;
        impl<'a> Visitor<'a> for TimeVisitor {
            type Value = Time;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("TimeMs as a string")
            }

            fn visit_str<E>(self, time: &str) -> StdResult<Self::Value, E>
            where
                E: de::Error,
            {
                let time_ms = time.parse().unwrap();
                Ok(Time(Duration::from_millis(time_ms)))
            }
        }

        deserializer.deserialize_any(TimeVisitor)
    }
}

impl Deref for Time {
    type Target = Duration;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Lyrics {
    pub fn parse_file(file: &PathBuf) -> Result<Self> {
        serde_json::from_reader(File::open(file).unwrap())
    }

    pub fn fix_end_times(&mut self) {
        if self.error {
            return;
        }

        let mut index = 0;
        loop {
            let Some(first) = self.lines.get(index) else {
                break;
            };
            let Some(second) = self.lines.get(index + 1) else {
                break;
            };

            if second.words == "♪" || second.words.is_empty() {
                first.endTimeMs.set(second.startTimeMs.get());
                self.lines.remove(index + 1);
                continue;
            }

            index += 1;
        }
    }
}

impl Time {
    pub fn is_valid(&self) -> bool {
        self.0.as_secs() > 0
    }
}
