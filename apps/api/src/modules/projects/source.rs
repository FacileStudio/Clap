use tronc::Error;

use super::error::ProjectError;

/// The largest JSON body of a route that carries a source: room for the default cap once every
/// byte is escaped, and for the envelope around it.
pub const BODY_MAX_BYTES: usize = 4 << 20;

/// What a new project holds when the request gives no source.
pub const STARTER: &str = "---\nsize: 1920x1080\nlayout: raw\nduration: 4\n---\n\n\
<div class=\"clip\" data-start=\"0\" data-duration=\"4\" data-track-index=\"1\">\n  <h1 id=\"title\">New project</h1>\n</div>\n\
<script>\n  scene.timeline((tl) => tl.from(\"#title\", { opacity: 0, y: 40, duration: 0.8 }));\n</script>\n";

/// Parses `source` as the scene format, within `max` bytes: `413` past the cap, the typed
/// problems with their lines when it does not parse.
pub fn checked(source: &str, max: usize) -> Result<scene::Project, ProjectError> {
    if source.len() > max {
        return Err(Error::too_large(format!("the source is over the {max}-byte limit")).into());
    }
    scene::parse(source).map_err(ProjectError::Invalid)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_source_is_capped_then_parsed() {
        let starter = checked(STARTER, 4096).unwrap();
        assert_eq!((starter.scenes().len(), starter.duration()), (1, 4.0));
        assert!(
            matches!(checked("# A\n", 2), Err(ProjectError::Api(error)) if error.code == tronc::Code::ResourceExhausted)
        );
        let Err(ProjectError::Invalid(errors)) = checked("![](https://x.test/a.png)\n", 100) else { panic!() };
        assert_eq!(errors.0[0].line, 1);
    }
}
