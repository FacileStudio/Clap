//! The worker's side of `DATA_DIR/renders`, which a compromised worker controls. Every path is
//! opened relative to the renders directory with `openat2`, refusing any symbolic link, magic
//! link or mount crossing on the way, and with `O_NONBLOCK` so a planted FIFO cannot hang the
//! API. What is opened is then checked on the descriptor itself: a regular file, with one link,
//! within a size cap. Nothing the worker wrote is renamed or served in place.

use std::fs::File;
use std::io::{self, Read, Write};
use std::os::fd::OwnedFd;
use std::path::Path;

use rustix::fs::{CWD, FileType, Mode, OFlags, ResolveFlags};

const RESOLVE: ResolveFlags = ResolveFlags::NO_SYMLINKS
    .union(ResolveFlags::BENEATH)
    .union(ResolveFlags::NO_MAGICLINKS)
    .union(ResolveFlags::NO_XDEV);

const SAFE: OFlags = OFlags::CLOEXEC.union(OFlags::NOFOLLOW).union(OFlags::NONBLOCK).union(OFlags::NOCTTY);

/// The mode of a job directory: the worker's group writes in it, and the sticky bit keeps it from
/// renaming or removing what the API put there; new entries keep the directory's group.
pub const JOB_MODE: u32 = 0o3775;

/// The mode of a bundle directory: the worker extracts frames under it.
pub const BUNDLE_MODE: u32 = 0o2775;

fn root(data_dir: &Path) -> io::Result<OwnedFd> {
    let flags = OFlags::RDONLY | OFlags::DIRECTORY | OFlags::CLOEXEC | OFlags::NOFOLLOW;
    Ok(rustix::fs::open(data_dir.join("renders"), flags, Mode::empty())?)
}

fn opened(data_dir: &Path, relative: &str, flags: OFlags, mode: u32) -> io::Result<OwnedFd> {
    let root = root(data_dir)?;
    Ok(rustix::fs::openat2(&root, relative, flags | SAFE, Mode::from_raw_mode(mode), RESOLVE)?)
}

/// The regular file at `relative`, opened for reading, and its size. Refused: a link anywhere on
/// the path, anything but a regular file, a file with another hard link, a file over `max`.
pub fn regular(data_dir: &Path, relative: &str, max: u64) -> io::Result<(File, u64)> {
    let fd = opened(data_dir, relative, OFlags::RDONLY, 0)?;
    let stat = rustix::fs::fstat(&fd)?;
    let refused = |why: &str| Err(io::Error::other(format!("{relative}: {why}")));
    if FileType::from_raw_mode(stat.st_mode) != FileType::RegularFile {
        return refused("not a regular file");
    }
    if stat.st_nlink != 1 {
        return refused("the file has another hard link");
    }
    match u64::try_from(stat.st_size) {
        Ok(size) if size <= max => Ok((File::from(fd), size)),
        _ => refused("the file is too large"),
    }
}

/// The bytes of the small regular file at `relative`, or `None` when it is missing or refused.
pub fn read(data_dir: &Path, relative: &str, max: u64) -> Option<Vec<u8>> {
    let (file, size) = regular(data_dir, relative, max).ok()?;
    let mut bytes = Vec::with_capacity(size as usize);
    file.take(max).read_to_end(&mut bytes).ok()?;
    Some(bytes)
}

/// Creates the empty file `relative`, or leaves it as it is: what the `cancel` signal needs.
/// Never truncates, never follows a link, never opens anything but a regular file.
pub fn touch(data_dir: &Path, relative: &str) -> io::Result<()> {
    let fd = opened(data_dir, relative, OFlags::WRONLY | OFlags::CREATE, 0o664)?;
    match FileType::from_raw_mode(rustix::fs::fstat(&fd)?.st_mode) {
        FileType::RegularFile => Ok(()),
        _ => Err(io::Error::other(format!("{relative}: not a regular file"))),
    }
}

/// Copies the worker's file `relative` into `target`, a path the API alone writes, through the
/// descriptor [`regular`] checked: a link swapped in after the check is never read. The copy
/// goes to a temporary name first. Answers the size.
pub fn copy_out(data_dir: &Path, relative: &str, target: &Path, max: u64) -> io::Result<u64> {
    let (source, size) = regular(data_dir, relative, max)?;
    let partial = target.with_extension("part");
    let copied = (|| {
        let mut written = File::options().write(true).create_new(true).open(&partial)?;
        let copied = io::copy(&mut source.take(size), &mut written)?;
        written.flush()?;
        written.sync_all()?;
        match copied == size {
            true => std::fs::rename(&partial, target).map(|()| size),
            false => Err(io::Error::other(format!("{relative} shrank while it was copied"))),
        }
    })();
    if copied.is_err() {
        std::fs::remove_file(&partial).ok();
    }
    copied
}

/// Publishes the job `name`: creates `renders/<name>` ([`JOB_MODE`]), moves the bundle the API
/// built in `staged`, a directory the worker cannot see, into it, then writes `job.json` under a
/// temporary name opened with `O_EXCL` and renames it, both through the job directory's
/// descriptor. The worker sees a job only once `job.json` is there.
pub fn publish(data_dir: &Path, name: &str, staged: &Path, job: &[u8]) -> io::Result<()> {
    let root = root(data_dir)?;
    rustix::fs::mkdirat(&root, name, Mode::from_raw_mode(0o700))?;
    let dir = rustix::fs::openat2(&root, name, OFlags::RDONLY | OFlags::DIRECTORY | SAFE, Mode::empty(), RESOLVE)?;
    rustix::fs::fchmod(&dir, Mode::from_raw_mode(JOB_MODE))?;
    rustix::fs::renameat(CWD, staged, &dir, "bundle")?;
    let flags = OFlags::WRONLY | OFlags::CREATE | OFlags::EXCL | SAFE;
    let temporary = rustix::fs::openat2(&dir, "job.json.tmp", flags, Mode::from_raw_mode(0o664), RESOLVE)?;
    let mut file = File::from(temporary);
    file.write_all(job)?;
    file.sync_all()?;
    Ok(rustix::fs::renameat(&dir, "job.json.tmp", &dir, "job.json")?)
}
