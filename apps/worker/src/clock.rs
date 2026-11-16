//! RFC 3339 timestamps.

use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

/// Now, in UTC, to the second.
pub(crate) fn now() -> String {
    let now = OffsetDateTime::now_utc().replace_nanosecond(0).unwrap_or(OffsetDateTime::UNIX_EPOCH);
    now.format(&Rfc3339).unwrap_or_default()
}

/// The instant `text` names.
pub(crate) fn parse(text: &str) -> Option<OffsetDateTime> {
    OffsetDateTime::parse(text.trim(), &Rfc3339).ok()
}

/// The seconds since `text`, or `None` when it is not a timestamp.
pub(crate) fn age(text: &str) -> Option<i64> {
    parse(text).map(|then| (OffsetDateTime::now_utc() - then).whole_seconds())
}
