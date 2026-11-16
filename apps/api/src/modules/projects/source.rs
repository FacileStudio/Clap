use tronc::Error;

use super::error::ProjectError;

/// The largest JSON body of a route that carries a source: room for the default cap once every
/// byte is escaped, and for the envelope around it.
pub const BODY_MAX_BYTES: usize = 4 << 20;

/// Parses `source` as the scene format, within `max` bytes: `413` past the cap, the typed
/// problems with their lines when it does not parse.
pub fn checked(source: &str, max: usize) -> Result<scene::Project, ProjectError> {
    if source.len() > max {
        return Err(Error::too_large(format!("the source is over the {max}-byte limit")).into());
    }
    clap_timeline::parse(source).map_err(ProjectError::Invalid)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_source_is_capped_then_parsed() {
        let starter = checked(&clap_timeline::starter("de"), 4096).unwrap();
        assert_eq!((starter.scenes().len(), starter.duration(), starter.config().lang.as_str()), (1, 4.0, "de"));
        assert!(
            matches!(checked("# A\n", 2), Err(ProjectError::Api(error)) if error.code == tronc::Code::ResourceExhausted)
        );
        let Err(ProjectError::Invalid(errors)) = checked("![](https://x.test/a.png)\n", 100) else { panic!() };
        assert_eq!(errors.0[0].line, 1);
    }
}
