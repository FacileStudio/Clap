pub use clap_timeline::{MAX_ASSET_NAME as MAX_NAME, asset_reference as referenced, valid_asset_name as valid};

/// The folder an upload lands in when it names none, as the format's examples write it.
pub const FOLDER: &str = "assets";

/// The asset name for an upload that gives only its file name: [`FOLDER`], then the base name
/// with every other character turned into `-`.
pub fn from_file(raw: &str) -> String {
    let base = raw.rsplit(['/', '\\']).next().unwrap_or_default();
    let kept = |c: char| if c.is_ascii_alphanumeric() || "._-".contains(c) { c } else { '-' };
    let clean: String = base.chars().map(kept).collect();
    let clean: String = clean.trim_start_matches(['.', '-']).chars().take(MAX_NAME - FOLDER.len() - 1).collect();
    format!("{FOLDER}/{}", if clean.is_empty() { "file" } else { &clean })
}

/// The name of an upload: the one `asked` for when it is a valid name, `400` when it is not,
/// and one made from `file_name` when the upload names none.
pub fn chosen(asked: Option<&str>, file_name: &str) -> tronc::Result<String> {
    let Some(asked) = asked.map(str::trim).filter(|asked| !asked.is_empty()) else {
        return Ok(from_file(file_name));
    };
    match valid(asked) {
        true => Ok(asked.to_owned()),
        false => Err(tronc::Error::invalid(
            "an asset name is a relative path of letters, digits, `.`, `_` and `-`, such as assets/clip.mp4",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_file_name_becomes_a_safe_name_and_a_bad_one_is_refused() {
        assert_eq!(from_file("chart.png"), "assets/chart.png");
        assert_eq!(from_file("../../etc/passwd"), "assets/passwd");
        assert_eq!(from_file("C:\\Users\\me\\Capture d'écran.png"), "assets/Capture-d--cran.png");
        assert_eq!(from_file(".env"), "assets/env");
        assert_eq!(from_file(""), "assets/file");
        assert!(valid(&from_file(&"é".repeat(400))));
        assert_eq!(chosen(Some(" a/b.mp4 "), "x").unwrap(), "a/b.mp4");
        assert_eq!(chosen(None, "x.mp4").unwrap(), "assets/x.mp4");
        assert!(chosen(Some("../x"), "x").is_err());
    }
}
