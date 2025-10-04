use std::path::Path;

use globset::{Glob, GlobMatcher};
use proc_macro2::Span;
use syn::LitStr;

use crate::parse::{
    ignore_attribute::IgnoreAttribute, legacy_ignore_config::LegacyIgnoreConfig,
    spanned::Spanned as _,
};

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
    pub fn new<P: AsRef<Path>>(
        legacy_config: &'config Option<LegacyIgnoreConfig>,
        ignore_args: &'config [IgnoreAttribute],
        current_dir: P,
    ) -> Result<Self, (Span, globset::Error)> {
        let globs = ignore_args.iter().map(|attr| {
            let full_path = current_dir.as_ref().join(attr.args.path.value());
            match Glob::new(full_path.to_str().expect("expected UTF-8")) {
                Ok(glob) => Ok(IgnoreGlob {
                    matcher: glob.compile_matcher(),
                    reason: attr.args.reason.as_ref(),
                }),
                Err(err) => Err((attr.args.path.span(), err)),
            }
        });

        let globs = if let Some(legacy_config) = legacy_config {
            legacy_config
                .paths()
                .paths()
                .iter()
                .map(|path| {
                    let full_path = current_dir.as_ref().join(path.path().value());
                    match Glob::new(full_path.to_str().expect("expected UTF-8")) {
                        Ok(glob) => Ok(IgnoreGlob {
                            matcher: glob.compile_matcher(),
                            reason: path.reason().as_ref(),
                        }),
                        Err(err) => Err((path.span(), err)),
                    }
                })
                .chain(globs)
                .collect::<Result<Vec<_>, _>>()?
        } else {
            globs.collect::<Result<Vec<_>, _>>()?
        };

        Ok(IgnoreMatcher {
            globs,
            default_reason: legacy_config.as_ref().and_then(|cfg| cfg.reason().as_ref()),
        })
    }
}

impl IgnoreMatcher<'_> {
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
