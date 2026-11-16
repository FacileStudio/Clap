/// How many leading bytes [`sniff`] needs at most to type a file.
pub const HEAD_BYTES: usize = 2048;

fn boxed(bytes: &[u8]) -> Option<&'static str> {
    match bytes.get(8..12)? {
        b"avif" | b"avis" => Some("image/avif"),
        b"M4A " | b"M4B " => Some("audio/mp4"),
        b"qt  " => Some("video/quicktime"),
        _ => Some("video/mp4"),
    }
}

fn svg(bytes: &[u8]) -> bool {
    let head = String::from_utf8_lossy(&bytes[..bytes.len().min(2048)]);
    let head = head.trim_start_matches('\u{feff}').trim_start();
    let opens = ["<svg", "<?xml", "<!--", "<!DOCTYPE svg", "<!doctype svg"].iter().any(|open| head.starts_with(open));
    opens && head.contains("<svg")
}

fn media(bytes: &[u8]) -> Option<&'static str> {
    match bytes {
        [_, _, _, _, b'f', b't', b'y', b'p', ..] => boxed(bytes),
        [0x1a, 0x45, 0xdf, 0xa3, ..] if bytes[..bytes.len().min(64)].windows(8).any(|w| w == b"matroska") => {
            Some("video/x-matroska")
        }
        [0x1a, 0x45, 0xdf, 0xa3, ..] => Some("video/webm"),
        [b'I', b'D', b'3', ..] | [0xff, 0xfb | 0xf3 | 0xf2, ..] => Some("audio/mpeg"),
        [0xff, 0xf1 | 0xf9, ..] => Some("audio/aac"),
        [b'f', b'L', b'a', b'C', ..] => Some("audio/flac"),
        [b'R', b'I', b'F', b'F', _, _, _, _, b'W', b'A', b'V', b'E', ..] => Some("audio/wav"),
        [b'O', b'g', b'g', b'S', ..] => Some("audio/ogg"),
        _ => None,
    }
}

fn font(bytes: &[u8]) -> Option<&'static str> {
    match bytes {
        [b'w', b'O', b'F', b'2', ..] => Some("font/woff2"),
        [b'w', b'O', b'F', b'F', ..] => Some("font/woff"),
        [0, 1, 0, 0, ..] => Some("font/ttf"),
        [b'O', b'T', b'T', b'O', ..] => Some("font/otf"),
        _ => None,
    }
}

fn image(bytes: &[u8]) -> Option<&'static str> {
    match bytes {
        [0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a, ..] => Some("image/png"),
        [0xff, 0xd8, 0xff, ..] => Some("image/jpeg"),
        [b'G', b'I', b'F', b'8', b'7' | b'9', b'a', ..] => Some("image/gif"),
        [b'R', b'I', b'F', b'F', _, _, _, _, b'W', b'E', b'B', b'P', ..] => Some("image/webp"),
        _ => svg(bytes).then_some("image/svg+xml"),
    }
}

/// The media type the bytes declare by their content, among those a scene loads: images, video,
/// audio and fonts. Anything else answers `None` and is refused; the type an upload declares is
/// never trusted. SVG is served with a `sandbox` policy, so opened on its own it runs nothing.
pub fn sniff(bytes: &[u8]) -> Option<&'static str> {
    image(bytes).or_else(|| media(bytes)).or_else(|| font(bytes))
}

/// The kind of a media type: `video`, `audio`, `image`, or `other` for a font.
pub fn kind(mime: &str) -> &'static str {
    match mime.split('/').next() {
        Some("video") => "video",
        Some("audio") => "audio",
        Some("image") => "image",
        _ => "other",
    }
}

#[cfg(test)]
mod tests {
    use super::{kind, sniff};

    const KNOWN: [(&[u8], &str); 18] = [
        (b"\x89PNG\r\n\x1a\nrest", "image/png"),
        (b"\xff\xd8\xff\xe0", "image/jpeg"),
        (b"GIF89a..", "image/gif"),
        (b"RIFF\0\0\0\0WEBPVP8 ", "image/webp"),
        (b"\0\0\0\x1cftypavif\0\0", "image/avif"),
        (b"<svg xmlns=\"http://www.w3.org/2000/svg\"/>", "image/svg+xml"),
        (b"\xef\xbb\xbf<?xml version=\"1.0\"?>\n<svg/>", "image/svg+xml"),
        (b"\0\0\0\x18ftypisom\0\0", "video/mp4"),
        (b"\0\0\0\x14ftypqt  \0\0", "video/quicktime"),
        (b"\0\0\0\x18ftypM4A \0\0", "audio/mp4"),
        (b"\x1a\x45\xdf\xa3....", "video/webm"),
        (b"\x1a\x45\xdf\xa3\x01\0\0\0\0\0\0\x23\x42\x82\x88matroska", "video/x-matroska"),
        (b"ID3\x04\0", "audio/mpeg"),
        (b"fLaC\0", "audio/flac"),
        (b"RIFF\0\0\0\0WAVEfmt ", "audio/wav"),
        (b"OggS\0", "audio/ogg"),
        (b"wOF2\0", "font/woff2"),
        (b"OTTO\0", "font/otf"),
    ];

    const REFUSED: [&[u8]; 6] =
        [b"<!doctype html><script>1</script>", b"<html><svg/></html>", b"alert(1)", b"%PDF-1.7", b"", b"PK\x03\x04"];

    #[test]
    fn media_are_recognised_by_their_bytes_and_everything_else_is_refused() {
        for (bytes, mime) in KNOWN {
            assert_eq!(sniff(bytes), Some(mime), "{mime}");
        }
        for bytes in REFUSED {
            assert_eq!(sniff(bytes), None, "{:?}", String::from_utf8_lossy(bytes));
        }
        let kinds = (kind("video/mp4"), kind("audio/flac"), kind("image/png"), kind("font/woff2"));
        assert_eq!(kinds, ("video", "audio", "image", "other"));
    }
}
