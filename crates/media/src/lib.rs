//! What `clap-api` and the render worker share about a media job: the `ffprobe` and `ffmpeg`
//! argument vectors, their limits, the files a job holds and the order they appear in. The API
//! writes the job and reads the files back; the worker, in its sandbox, runs the commands. Both
//! sides build every command and every plan from here, so they cannot drift.
#![deny(unused, dead_code, unsafe_code)]

mod argv;
mod job;
mod limits;
mod peaks;
mod probe;
mod strip;

pub use argv::{DECODE, argv, framed, poster_args, probe_args, proxy_args};
pub use job::{Make, Step, files, max};
pub use limits::{POSTER_TIMEOUT, PROBE_TIMEOUT, PROXY_TIMEOUT, poster_at, proxy_limit};
pub use peaks::{BUCKETS, RATE, buckets, checked, measure, peaks_args};
pub use probe::{Probe, parse};
pub use strip::{HEIGHT, MAX_TILES, MIN_STEP_MS, Plan, plan, strip_args};
