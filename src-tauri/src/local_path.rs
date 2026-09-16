use std::path::PathBuf;

pub fn from_dialog(value: &str) -> Result<PathBuf, String> {
    if !value.starts_with("file:") {
        return Ok(PathBuf::from(value));
    }
    tauri::Url::parse(value)
        .map_err(|error| format!("invalid file URL: {error}"))?
        .to_file_path()
        .map_err(|()| "invalid local file URL".to_owned())
}

pub fn string_from_dialog(value: &str) -> Result<String, String> {
    from_dialog(value)?
        .into_os_string()
        .into_string()
        .map_err(|_| "selected file path is not valid UTF-8".to_owned())
}

#[cfg(test)]
mod tests {
    use super::{from_dialog, string_from_dialog};

    #[test]
    fn ios_file_urls_become_local_paths() {
        assert_eq!(
            from_dialog("file:///private/var/mobile/My%20File.csv").unwrap(),
            std::path::PathBuf::from("/private/var/mobile/My File.csv")
        );
    }

    #[test]
    fn desktop_paths_are_unchanged() {
        assert_eq!(
            string_from_dialog("/Users/example/list.csv").unwrap(),
            "/Users/example/list.csv"
        );
    }
}
