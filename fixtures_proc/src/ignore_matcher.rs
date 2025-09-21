use std::path::Path;

use globset::{Glob, GlobMatcher};
use syn::LitStr;

use crate::parse::ignore_config::{IgnoreConfig, IgnorePath};

struct IgnoreGlob<'config> {
    matcher: GlobMatcher,
    reason: Option<&'config LitStr>,
}

pub struct IgnoreMatcher<'config> {
    globs: Vec<IgnoreGlob<'config>>,
    default_reason: Option<&'config LitStr>,
}

#[derive(Debug)]
pub enum MatchResult<'config> {
    Matched { reason: Option<&'config LitStr> },
    Unmatched,
}

impl<'config> IgnoreMatcher<'config> {
    pub fn new(
        config: &'config IgnoreConfig,
    ) -> Result<Self, (&'config IgnorePath, globset::Error)> {
        let globs = config
            .paths()
            .paths()
            .iter()
            .map(|path| {
                let cwd = std::env::current_dir().unwrap();
                let full_path = cwd.join(path.path().value());
                match Glob::new(full_path.to_str().expect("expected UTF-8")) {
                    Ok(glob) => Ok(IgnoreGlob {
                        matcher: glob.compile_matcher(),
                        reason: path.reason().as_ref(),
                    }),
                    Err(err) => Err((path, err)),
                }
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(IgnoreMatcher {
            globs,
            default_reason: config.reason().as_ref(),
        })
    }
}

impl IgnoreMatcher<'_> {
    pub fn empty() -> Self {
        IgnoreMatcher {
            globs: vec![],
            default_reason: None,
        }
    }

    pub fn matched<P: AsRef<Path>>(&self, path: P) -> MatchResult<'_> {
        if self.globs.is_empty() {
            return MatchResult::Unmatched;
        }

        for glob in &self.globs {
            if glob.matcher.is_match(&path) {
                return MatchResult::Matched {
                    reason: glob
                        .reason
                        .as_ref()
                        .or(self.default_reason.as_ref())
                        .map(|v| &**v),
                };
            }
        }

        MatchResult::Unmatched
    }
}
