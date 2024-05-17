use std::collections::HashSet;
use std::fs::File;
use std::path::Path;

use std::io::{self, BufRead, BufReader};

use ffmpeg_sidecar::paths::sidecar_dir;

pub fn ffmpeg_path_as_str() -> Result<String, String> {
    let binary_name = if cfg!(target_os = "windows") {
        "ffmpeg.exe"
    } else {
        "ffmpeg"
    };

    let path = sidecar_dir().map_err(|e| e.to_string())?.join(binary_name);

    if Path::new(&path).exists() {
        path.to_str()
            .map(|s| s.to_owned())
            .ok_or_else(|| "Failed to convert FFmpeg binary path to string".to_string())
    } else {
        Ok("ffmpeg".to_string())
    }
}

pub fn load_segment_list(segment_list_path: &Path) -> io::Result<HashSet<String>> {
    let file = File::open(segment_list_path)?;
    let reader = BufReader::new(file);

    let mut segments = HashSet::new();
    for line_result in reader.lines() {
        let line = line_result?;
        if !line.is_empty() {
            segments.insert(line);
        }
    }

    Ok(segments)
}
