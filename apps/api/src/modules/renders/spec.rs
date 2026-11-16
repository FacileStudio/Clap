use tronc::{Error, Result};

use super::types::RenderRequest;
use crate::limits::RenderLimits;

/// The frame rate of a render that names none.
pub const DEFAULT_FPS: u32 = 30;

/// What one render produces, checked against the caps.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Spec {
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    /// `(start, end)` in seconds, or `None` for the whole project.
    pub range: Option<(f64, f64)>,
    /// How many frames that is.
    pub frames: i64,
}

fn sized(width: u32, height: u32, limits: &RenderLimits) -> Result<()> {
    let (long, short) = (width.max(height), width.min(height));
    if short < 16 || !width.is_multiple_of(2) || !height.is_multiple_of(2) {
        return Err(Error::invalid(format!("a render is at least 16 pixels a side, both even, not {width}x{height}")));
    }
    match long > limits.width || short > limits.height {
        true => Err(Error::invalid(format!(
            "a render is at most {}x{}, not {width}x{height}: pass a smaller width and height",
            limits.width, limits.height
        ))),
        false => Ok(()),
    }
}

fn ranged(range: Option<[f64; 2]>, duration: f64) -> Result<Option<(f64, f64)>> {
    let Some([start, end]) = range else { return Ok(None) };
    let inside = start.is_finite() && end.is_finite() && start >= 0.0 && start < end && end <= duration + 1e-6;
    match inside {
        true => Ok(Some((start, end.min(duration)))),
        false => Err(Error::invalid(format!(
            "range is [start, end] in seconds with 0 <= start < end <= {duration}, the project's length"
        ))),
    }
}

/// The render `request` asks for of a project that is `size` pixels and `duration` seconds:
/// the project's own size, 30 frames a second and the whole length unless it says otherwise.
/// `400` past a cap.
pub fn spec(request: &RenderRequest, size: (u32, u32), duration: f64, limits: &RenderLimits) -> Result<Spec> {
    let (width, height) = (request.width.unwrap_or(size.0), request.height.unwrap_or(size.1));
    sized(width, height, limits)?;
    let fps = request.fps.unwrap_or(DEFAULT_FPS.min(limits.fps));
    if fps == 0 || fps > limits.fps {
        return Err(Error::invalid(format!("fps is 1 to {}, not {fps}", limits.fps)));
    }
    let range = ranged(request.range, duration)?;
    let seconds = range.map_or(duration, |(start, end)| end - start);
    if seconds > f64::from(limits.seconds) {
        return Err(Error::invalid(format!(
            "a render lasts at most {} s and this one lasts {seconds:.1} s: pass a shorter range",
            limits.seconds
        )));
    }
    let frames = (seconds * f64::from(fps)).round().max(1.0) as i64;
    Ok(Spec { width, height, fps, range, frames })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ask(width: Option<u32>, height: Option<u32>, fps: Option<u32>, range: Option<[f64; 2]>) -> RenderRequest {
        RenderRequest { width, height, fps, range }
    }

    #[test]
    fn a_bare_request_renders_the_whole_project_at_its_own_size() {
        let spec = spec(&RenderRequest::default(), (1280, 720), 6.0, &RenderLimits::default()).unwrap();
        assert_eq!(spec, Spec { width: 1280, height: 720, fps: 30, range: None, frames: 180 });
        let portrait = ask(Some(1080), Some(1920), Some(24), Some([1.0, 3.5]));
        let spec = super::spec(&portrait, (1280, 720), 6.0, &RenderLimits::default()).unwrap();
        assert_eq!((spec.width, spec.height, spec.range, spec.frames), (1080, 1920, Some((1.0, 3.5)), 60));
    }

    #[test]
    fn a_request_past_a_cap_is_refused() {
        let limits = RenderLimits::default();
        let refused = [
            ask(Some(3840), Some(2160), None, None),
            ask(Some(1921), Some(1080), None, None),
            ask(Some(1281), None, None, None),
            ask(Some(8), Some(8), None, None),
            ask(None, None, Some(60), None),
            ask(None, None, Some(0), None),
            ask(None, None, None, Some([3.0, 2.0])),
            ask(None, None, None, Some([0.0, 7.0])),
            ask(None, None, None, Some([f64::NAN, 2.0])),
        ];
        for request in refused {
            let error = spec(&request, (1280, 720), 6.0, &limits).unwrap_err();
            assert_eq!(error.code, tronc::Code::InvalidArgument, "{request:?}");
        }
        assert!(spec(&RenderRequest::default(), (1280, 720), 601.0, &limits).unwrap_err().message.contains("600 s"));
        assert!(spec(&ask(None, None, None, Some([0.0, 600.0])), (1280, 720), 900.0, &limits).is_ok());
    }
}
