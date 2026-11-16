//! The caps one instance holds its projects, uploads and renders to, all tunable by environment.

use tronc::env::{EnvError, Vars};

/// The largest project source, in bytes, unless `SOURCE_MAX_BYTES` says otherwise.
pub const SOURCE_MAX_BYTES: u64 = 512 << 10;

/// The largest single asset, in bytes, unless `ASSET_MAX_BYTES` says otherwise: 2 GiB.
pub const ASSET_MAX_BYTES: u64 = 2 << 30;

/// The most bytes the assets of one space, or of one person's own projects, hold together,
/// unless `SPACE_ASSETS_MAX_BYTES` says otherwise: 20 GiB.
pub const SPACE_ASSETS_MAX_BYTES: u64 = 20 << 30;

/// The most bytes a posted render bundle holds, unless `RENDER_BUNDLE_MAX_BYTES` says otherwise:
/// 1 GiB.
pub const RENDER_BUNDLE_MAX_BYTES: u64 = 1 << 30;

/// The most files a posted render bundle holds.
pub const RENDER_BUNDLE_MAX_FILES: usize = 2000;

/// What one render may ask for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RenderLimits {
    /// `RENDER_MAX_WIDTH`: the longest side, in pixels.
    pub width: u32,
    /// `RENDER_MAX_HEIGHT`: the shortest side, in pixels.
    pub height: u32,
    /// `RENDER_MAX_FPS`: the highest frame rate.
    pub fps: u32,
    /// `RENDER_MAX_SECONDS`: the longest rendered duration.
    pub seconds: u32,
    /// `RENDER_QUEUE_MAX`: how many renders may wait or run at once, on the whole instance.
    pub queue: u32,
}

impl Default for RenderLimits {
    fn default() -> Self {
        RenderLimits { width: 1920, height: 1080, fps: 30, seconds: 600, queue: 10 }
    }
}

/// Every cap of the instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Limits {
    /// `SOURCE_MAX_BYTES`: the largest project source.
    pub source: usize,
    /// `ASSET_MAX_BYTES`: the largest single asset.
    pub asset: u64,
    /// `SPACE_ASSETS_MAX_BYTES`: the most bytes the assets of one space hold together.
    pub space_assets: u64,
    /// `RENDER_BUNDLE_MAX_BYTES`: the most bytes a posted render bundle holds.
    pub bundle: u64,
    /// The render caps.
    pub render: RenderLimits,
}

fn positive(vars: &Vars, key: &str, default: u64) -> Result<u64, EnvError> {
    let raw = vars.int(key, default as i64)?;
    match u64::try_from(raw) {
        Ok(value) if value > 0 => Ok(value),
        _ => Err(EnvError(format!("{key} must be a positive whole number, not {raw}"))),
    }
}

fn small(vars: &Vars, key: &str, default: u32) -> Result<u32, EnvError> {
    let value = positive(vars, key, u64::from(default))?;
    u32::try_from(value).map_err(|_| EnvError(format!("{key} is too large: {value}")))
}

impl RenderLimits {
    /// Reads the `RENDER_*` variables.
    pub fn from_vars(vars: &Vars) -> Result<Self, EnvError> {
        let default = RenderLimits::default();
        Ok(RenderLimits {
            width: small(vars, "RENDER_MAX_WIDTH", default.width)?,
            height: small(vars, "RENDER_MAX_HEIGHT", default.height)?,
            fps: small(vars, "RENDER_MAX_FPS", default.fps)?,
            seconds: small(vars, "RENDER_MAX_SECONDS", default.seconds)?,
            queue: small(vars, "RENDER_QUEUE_MAX", default.queue)?,
        })
    }
}

impl Limits {
    /// Reads every cap, refusing zero, a negative number and anything that is not a number.
    pub fn from_vars(vars: &Vars) -> Result<Self, EnvError> {
        let source = positive(vars, "SOURCE_MAX_BYTES", SOURCE_MAX_BYTES)?;
        Ok(Limits {
            source: usize::try_from(source).map_err(|_| EnvError("SOURCE_MAX_BYTES is too large".to_owned()))?,
            asset: positive(vars, "ASSET_MAX_BYTES", ASSET_MAX_BYTES)?,
            space_assets: positive(vars, "SPACE_ASSETS_MAX_BYTES", SPACE_ASSETS_MAX_BYTES)?,
            bundle: positive(vars, "RENDER_BUNDLE_MAX_BYTES", RENDER_BUNDLE_MAX_BYTES)?,
            render: RenderLimits::from_vars(vars)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vars(extra: &[(&str, &str)]) -> Vars {
        Vars::from_pairs(extra.to_vec())
    }

    #[test]
    fn the_defaults_are_two_gib_a_file_twenty_a_space_and_1080p30_for_ten_minutes() {
        let limits = Limits::from_vars(&vars(&[])).unwrap();
        assert_eq!((limits.source, limits.asset, limits.space_assets), (512 << 10, 2 << 30, 20 << 30));
        assert_eq!(limits.bundle, 1 << 30);
        assert_eq!(limits.render, RenderLimits { width: 1920, height: 1080, fps: 30, seconds: 600, queue: 10 });
    }

    #[test]
    fn every_cap_is_tunable_and_must_be_positive() {
        let limits = Limits::from_vars(&vars(&[("ASSET_MAX_BYTES", "1024"), ("RENDER_MAX_FPS", "60")])).unwrap();
        assert_eq!((limits.asset, limits.render.fps), (1024, 60));
        for bad in [("SOURCE_MAX_BYTES", "0"), ("ASSET_MAX_BYTES", "-1"), ("RENDER_MAX_SECONDS", "lots")] {
            assert!(Limits::from_vars(&vars(&[bad])).is_err(), "{bad:?}");
        }
    }
}
