use sndfile::*;
use std::ffi::OsStr;
use std::path::Path;

/// This structure represents metadata of an Audio file
#[derive(Debug, Clone)]
pub struct AudioMeta {
    pub title: String,
    pub album: String,
    pub artist: String,
}

/// Identifies an audio file format
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

/// Generates an [`AudioFile`](AudioFile) structure by reading
/// an audio file.
///
/// # Arguments
/// * `file` - A [`String`](String) containing the path to the audio file.
///
/// ## Panics
/// If the given path to the audio file is invalid, this will panic.
pub fn get_audio_info(file: &String) -> AudioFile {
    let mut snd: SndFile = _open_file(file);
    let samplerate: usize = snd.get_samplerate();
    let n_channels = snd.get_channels();
    let n_frame = snd.len().unwrap();
    let fmt = _get_fmt(file);

    AudioFile {
        file_name: file.clone(),
        format: fmt,
        length: n_frame as f64 / samplerate as f64,
        sample_rate: samplerate,
        stereo: n_channels > 1,
        lossless: matches!(fmt, AudioFormat::FLAC | AudioFormat::WAV),
        metadata: _get_meta(&snd),
    }
}

/// Opens an audio file with [`sndfile`](sndfile)
///
/// # Arguments
/// * `file` - A [`String`](String) containing the path to the audio file.
///
/// ## Panics
/// If the given path to the audio file is invalid, this will panic.
fn _open_file(file: &String) -> SndFile {
    sndfile::OpenOptions::ReadOnly(ReadOptions::Auto)
        .from_path(file)
        .unwrap()
}

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
fn _get_fmt(file: &String) -> AudioFormat {
    let ext = _get_extension(file);
    match ext.to_lowercase().as_str() {
        "flac" => AudioFormat::FLAC,
        "wav" => AudioFormat::WAV,
        "ogg" => AudioFormat::OGG,
        _ => panic!("_get_fmt() failed"),
    }
}

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
fn _get_meta(sndfile: &SndFile) -> AudioMeta {
    AudioMeta {
        title: sndfile
            .get_tag(TagType::Title)
            .unwrap_or_else(|| "Unknown".to_owned()),
        album: sndfile
            .get_tag(TagType::Album)
            .unwrap_or_else(|| "Unknown".to_owned()),
        artist: sndfile
            .get_tag(TagType::Artist)
            .unwrap_or_else(|| "Unknown".to_owned()),
    }
}

/// Returns the extension of the given file.  
/// Used by [`_get_fmt`](_get_fmt).
///
/// # Arguments
/// * `file` - A [`String`](String) containing the path to the audio file.
///
/// ### Notes
/// The file extension is converted to lowercase.
fn _get_extension(file: &String) -> String {
    let res = Path::new(file)
        .extension()
        .and_then(OsStr::to_str)
        .expect("Invalid file name")
        .to_lowercase();
    res
}
