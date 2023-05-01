use serde::{
    de::{self, Deserialize as DeserializeTrait, Deserializer},
    Deserialize,
};
use serde_json::Result;
use std::{cell::Cell, fs::File, path::PathBuf, result::Result as StdResult, time::Duration};

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
    #[serde(deserialize_with = "parse_duration")]
    pub startTimeMs: Cell<Duration>,
    pub words: String,
    #[serde(deserialize_with = "parse_duration")]
    pub endTimeMs: Cell<Duration>,
}

fn parse_duration<'de, D>(deserializer: D) -> StdResult<Cell<Duration>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let number: u64 = s.parse().map_err(|e| de::Error::custom(format!("{e}")))?;
    Ok(Cell::new(Duration::from_millis(number)))
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

impl LyricsEntry {
    pub fn is_endtime_valid(&self) -> bool {
        self.endTimeMs.get().as_secs() > 0
    }
}
