//! What is on its way in right now: how many uploads each person is sending, and the bytes those
//! uploads have reserved in each space. It lives in this process, like the event broadcast.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use tronc::{Error, Result};

/// What a multipart request carries besides the file: boundaries, part headers, the `name` part.
/// A request this much longer than what is left still fits, and its stream stops at what is left.
pub const FRAMING: u64 = 64 << 10;

#[derive(Debug, Default)]
struct Book {
    users: HashMap<i64, usize>,
    spaces: HashMap<String, u64>,
}

/// The book every upload and posted bundle signs in to.
#[derive(Debug, Clone, Default)]
pub struct Inflight(Arc<Mutex<Book>>);

/// One upload in flight. Dropping it gives the slot and the reserved bytes back.
#[derive(Debug)]
pub struct Slot {
    book: Inflight,
    user: i64,
    space: String,
    bytes: u64,
}

impl Slot {
    /// The most bytes this upload may write.
    pub fn bytes(&self) -> u64 {
        self.bytes
    }
}

impl Drop for Slot {
    fn drop(&mut self) {
        let mut book = self.book.0.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(count) = book.users.get_mut(&self.user) {
            *count = count.saturating_sub(1);
        }
        book.users.retain(|_, count| *count > 0);
        if let Some(reserved) = book.spaces.get_mut(&self.space) {
            *reserved = reserved.saturating_sub(self.bytes);
        }
        book.spaces.retain(|_, reserved| *reserved > 0);
    }
}

/// What one upload asks for: who sends it, into which space, how long the request says it is,
/// and the caps it is held to.
#[derive(Debug, Clone, Copy)]
pub struct Ask<'a> {
    pub user: i64,
    pub per_user: usize,
    pub space: &'a str,
    pub length: Option<u64>,
    pub file_max: u64,
    pub room: u64,
}

impl Inflight {
    /// Signs one upload in, before a byte is read: `429` when the person already sends
    /// `per_user`, `413` when the request is longer than what the space has left once the
    /// uploads in flight are counted, [`FRAMING`] aside. Without a length, the upload reserves what is left, at
    /// most `file_max`.
    pub fn enter(&self, ask: Ask<'_>) -> Result<Slot> {
        let mut book = self.0.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        if book.users.get(&ask.user).copied().unwrap_or(0) >= ask.per_user {
            return Err(Error::rate_limited(format!(
                "you are already sending {} files: wait until one has finished",
                ask.per_user
            )));
        }
        let available = ask.room.saturating_sub(book.spaces.get(ask.space).copied().unwrap_or(0));
        let bytes = ask.length.unwrap_or(available).min(available).min(ask.file_max);
        if bytes == 0 || ask.length.is_some_and(|length| length > available.saturating_add(FRAMING)) {
            return Err(Error::too_large(format!("the space has {} MiB left for uploads", available >> 20)));
        }
        *book.users.entry(ask.user).or_default() += 1;
        *book.spaces.entry(ask.space.to_owned()).or_default() += bytes;
        Ok(Slot { book: self.clone(), user: ask.user, space: ask.space.to_owned(), bytes })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MIB: u64 = 1 << 20;

    fn ask(user: i64, length: Option<u64>) -> Ask<'static> {
        Ask {
            user,
            per_user: 2,
            space: "s1",
            length: length.map(|mib| mib * MIB),
            file_max: 100 * MIB,
            room: 150 * MIB,
        }
    }

    #[test]
    fn a_person_sends_two_files_at_once_and_a_space_never_reserves_past_its_room() {
        let book = Inflight::default();
        let first = book.enter(ask(1, Some(60))).unwrap();
        let second = book.enter(ask(1, Some(60))).unwrap();
        assert_eq!((first.bytes(), second.bytes()), (60 * MIB, 60 * MIB));
        assert_eq!(book.enter(ask(1, Some(1))).unwrap_err().code, tronc::Code::RateLimited);
        assert_eq!(book.enter(ask(2, Some(31))).unwrap_err().code, tronc::Code::ResourceExhausted);
        let rest = book.enter(ask(2, None)).unwrap();
        assert_eq!(rest.bytes(), 30 * MIB);
        assert_eq!(book.enter(ask(3, None)).unwrap_err().code, tronc::Code::ResourceExhausted);
        drop((first, rest));
        assert_eq!(book.enter(ask(1, Some(500))).unwrap_err().code, tronc::Code::ResourceExhausted);
        let capped = book.enter(ask(3, None)).unwrap();
        assert_eq!(capped.bytes(), 90 * MIB);
        drop((second, capped));
        let framed = Ask { length: Some(150 * MIB + 1000), file_max: 200 * MIB, ..ask(1, None) };
        assert_eq!(book.enter(framed).unwrap().bytes(), 150 * MIB, "the framing fits, the stream stops at the room");
        let left = book.0.lock().unwrap();
        assert!(left.users.is_empty() && left.spaces.is_empty());
    }
}
