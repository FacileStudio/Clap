//! When the asset links of a frame stop working. Links signed in the same ten minutes share one
//! expiry, so a document reloaded after an edit asks for the same URLs and the browser reuses
//! the files it already holds instead of fetching and decoding them again.

use super::TTL;

/// The span whose frames share one expiry.
pub const BUCKET: u64 = 10 * 60;

/// The expiry of the links signed at `now`: at least [`TTL`] away, the same for every `now` in
/// one [`BUCKET`].
pub fn expiry(now: u64) -> u64 {
    (now / BUCKET + 1) * BUCKET + TTL.as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn links_signed_in_one_bucket_share_an_expiry_at_least_a_ttl_away() {
        for now in [0, 1, 599, 600, 1_763_300_000, 1_763_300_599] {
            let at = expiry(now);
            assert!(at >= now + TTL.as_secs(), "{now}");
            assert!(at <= now + TTL.as_secs() + BUCKET, "{now}");
            assert_eq!(at, expiry(now / BUCKET * BUCKET), "{now}");
        }
        assert_ne!(expiry(599), expiry(600));
    }
}
