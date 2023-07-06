use sndfile::*;
use std::path::Path;

/// This structure represents metadata of an Audio file
#[derive(Debug, Clone)]
pub struct AudioMeta {
    pub title: String,
    pub album: String,
    pub artist: String,
}

/// Identifies an audio file format
#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AudioFormat {
    /// Free Lossless Audio Codec
    FLAC,
    /// Wave file
    WAV,
    /// Ogg Vorbis
    OGG,
}

/// This structure represents an Audio file
#[derive(Debug, Clone)]
pub struct AudioFile {
    /// Name of the file (for now this will contain the entire path)
    pub file_name: String,
    /// File format
    pub format: AudioFormat,
    /// Length of the track in seconds
    pub length: f64,
    /// Sample rate
    pub sample_rate: usize,
    /// Whether the audio track is stereo
    /// This is only true if the number if channels is 2
    pub stereo: bool,
    /// Whether the audio file is in a lossless format
    /// This is only `true` if `format` is [`AudioFormat::FLAC`](AudioFormat::FLAC) or [`AudioFormat::WAV`](AudioFormat::FLAC)
    pub lossless: bool,
    /// Metadata
    pub metadata: AudioMeta,
}

impl AudioFile {
    /// Generates an [`AudioFile`](AudioFile) structure by reading
    /// an audio file.
    ///
    /// # Arguments
    /// * `file` - A [`String`](String) containing the path to the audio file.
    ///
    /// ## Panics
    /// If the given path to the audio file is invalid, this will panic.
    pub fn new(file: &str) -> Self {
        let mut snd = Self::open_file(file);
        let samplerate: usize = snd.get_samplerate();
        let n_frame = snd.len().unwrap();
        let fmt = AudioFormat::from_path(file).expect("Failed to parse format");

        Self {
            file_name: file.to_string(),
            format: fmt,
            length: n_frame as f64 / samplerate as f64,
            sample_rate: samplerate,
            stereo: snd.get_channels() > 1,
            lossless: fmt.is_lossless(),
            metadata: snd.into(),
        }
    }

    /// Opens an audio file with [`sndfile`](sndfile)
    ///
    /// # Arguments
    /// * `file` - A [`String`](String) containing the path to the audio file.
    ///
    /// ## Panics
    /// If the given path to the audio file is invalid, this will panic.
    fn open_file(file: &str) -> SndFile {
        sndfile::OpenOptions::ReadOnly(ReadOptions::Auto)
            .from_path(file)
            .unwrap()
    }
}

impl AudioFormat {
    /// Gets the file format of the given audio file by checking
    /// it's file extension, then returns an enum value from [`AudioFormat`](AudioFormat).
    ///
    /// # Arguments
    /// * `file` - A [`String`](String) containing the path to the audio file.
    ///
    /// ## Panics
    /// If the file has an extension other than `.wav`, `.flac` or `.ogg` this will panic.
    ///
    /// ### Notes
    /// This function is __not__ case-sensitive, as the given file path is converted to
    /// lowercase, before it's compared.
    pub fn from_path(path: &str) -> Result<Self, ()> {
        let ext = Path::new(path).extension().unwrap().to_string_lossy();

        match ext.to_lowercase().as_str() {
            "flac" => Ok(AudioFormat::FLAC),
            "wav" => Ok(AudioFormat::WAV),
            "ogg" => Ok(AudioFormat::OGG),
            _ => Err(()),
        }
    }

    pub fn is_lossless(&self) -> bool {
        matches!(self, AudioFormat::FLAC | AudioFormat::WAV)
    }
}

impl std::fmt::Display for AudioFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::FLAC => "FLAC",
                Self::OGG => "OGG",
                Self::WAV => "WAV",
            }
        )
    }
}

impl Into<AudioMeta> for SndFile {
    /// Gets the necessary metadata from an opened audio file ([`SndFile`](SndFile)).  
    /// It'll read: `Title` ([`TagType::Title`](TagType::Title)),
    ///             `Album` ([`TagType::Album`](TagType::Album)) and
    ///             `Artist` ([`TagType::Artist`](TagType::Artist))
    ///
    /// # Arguments
    /// * `sndfile` - An opened audio file ([`SndFile`](SndFile)).
    ///
    /// ## Panics
    /// Depends on [`SndFile::get_tag()`](SndFile::get_tag())
    ///
    /// ### Notes
    /// In case the read tag is not defined, `"Unknown"` is used as a placeholder.
    fn into(self) -> AudioMeta {
        AudioMeta {
            title: self.get_tag(TagType::Title).unwrap_or("Unknown".to_owned()),
            album: self.get_tag(TagType::Album).unwrap_or("Unknown".to_owned()),
            artist: self
                .get_tag(TagType::Artist)
                .unwrap_or("Unknown".to_owned()),
        }
    }
}
