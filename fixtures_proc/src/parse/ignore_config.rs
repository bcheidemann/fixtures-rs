use proc_macro2::Span;
use quote::ToTokens;
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Ident, LitStr, Token,
};

use crate::parse::assignment::Assignment;

use super::spanned::Spanned;

pub struct IgnoreConfig {
    /// The span enclosing the ignore config
    span: Span,
    /// Path globs to be ignored.
    paths: IgnorePaths,
    /// Default ignore reason.
    reason: Option<LitStr>,
}

impl Parse for IgnoreConfig {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.step(|cursor| match cursor.token_tree() {
            Some((proc_macro2::TokenTree::Group(group), next)) => {
                Ok((Self::parse_group(group)?, next))
            }
            Some((proc_macro2::TokenTree::Literal(lit), next)) => {
                Ok((Self::parse_literal(lit)?, next))
            }
            _ => Err(cursor.error("Expected list, object, or string literal.")),
        })
    }
}

impl Spanned for IgnoreConfig {
    fn span(&self) -> Span {
        self.span
    }
}

impl IgnoreConfig {
    pub fn paths(&self) -> &IgnorePaths {
        &self.paths
    }

    pub fn reason(&self) -> &Option<LitStr> {
        &self.reason
    }

    fn parse_group(group: proc_macro2::Group) -> syn::Result<Self> {
        match group.delimiter() {
            proc_macro2::Delimiter::Brace => syn::parse::Parser::parse2(
                |input: ParseStream| IgnoreConfig::parse_object(input, group.span()),
                group.stream(),
            ),
            proc_macro2::Delimiter::Bracket => syn::parse::Parser::parse2(
                |input: ParseStream| IgnoreConfig::parse_list(input, group.span()),
                group.stream(),
            ),
            _ => Err(syn::Error::new(
                group.delim_span().span(),
                "Unexpected delimiter. Expected brace, bracket, or string literal.",
            )),
        }
    }

    fn parse_object(input: ParseStream, span: Span) -> syn::Result<Self> {
        let mut paths = None;
        let mut reason = None;

        while !input.is_empty() {
            let ident = input.parse::<Ident>()?;
            input.parse::<Token![=]>()?;
            match ident.to_string().as_str() {
                "paths" => {
                    if paths.is_some() {
                        return Err(syn::Error::new(ident.span(), "Duplicate assignment."));
                    }
                    paths = Some(input.parse()?);
                }
                "reason" => {
                    if reason.is_some() {
                        return Err(syn::Error::new(ident.span(), "Duplicate assignment."));
                    }
                    reason = Some(input.parse()?);
                }
                _ => {
                    return Err(syn::Error::new(
                        ident.span(),
                        "Invalid field identifier. Expected 'paths' or 'reason'.",
                    ))
                }
            }
            if input.is_empty() {
                break;
            }
            input.parse::<Token![,]>()?;
        }

        Ok(IgnoreConfig {
            span,
            paths: paths.ok_or_else(|| syn::Error::new(span, "The 'paths' field is missing."))?,
            reason,
        })
    }

    fn parse_list(input: ParseStream, span: Span) -> syn::Result<Self> {
        let ignore_paths = Punctuated::<IgnorePath, Token![,]>::parse_terminated(input)?;
        Ok(IgnoreConfig {
            span,
            paths: IgnorePaths {
                span,
                paths: ignore_paths.into_iter().collect(),
            },
            reason: None,
        })
    }

    fn parse_literal(lit: proc_macro2::Literal) -> syn::Result<Self> {
        let tokens = lit.to_token_stream();
        let lit_str: LitStr = syn::parse2(tokens)?;
        Ok(IgnoreConfig {
            span: lit_str.span(),
            paths: IgnorePaths::from_lit_str(lit_str),
            reason: None,
        })
    }
}

pub struct IgnorePaths {
    span: Span,
    paths: Vec<IgnorePath>,
}

impl Parse for IgnorePaths {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.step(|cursor| match cursor.token_tree() {
            Some((proc_macro2::TokenTree::Group(group), next)) => {
                Ok((Self::parse_group(group)?, next))
            }
            Some((proc_macro2::TokenTree::Literal(lit), next)) => {
                Ok((Self::parse_literal(lit)?, next))
            }
            _ => Err(cursor.error("Expected list, object or string literal.")),
        })
    }
}

impl Spanned for IgnorePaths {
    fn span(&self) -> Span {
        self.span
    }
}

impl IgnorePaths {
    pub fn paths(&self) -> &Vec<IgnorePath> {
        &self.paths
    }

    fn parse_group(group: proc_macro2::Group) -> syn::Result<Self> {
        match group.delimiter() {
            proc_macro2::Delimiter::Brace => syn::parse::Parser::parse2(
                |input: ParseStream| IgnorePaths::parse_object(input, group.span()),
                group.stream(),
            ),
            proc_macro2::Delimiter::Bracket => syn::parse::Parser::parse2(
                |input: ParseStream| IgnorePaths::parse_list(input, group.span()),
                group.stream(),
            ),
            _ => Err(syn::Error::new(
                group.delim_span().span(),
                "Unexpected delimiter. Expected bracket or string literal.",
            )),
        }
    }

    fn parse_object(input: ParseStream, span: Span) -> syn::Result<Self> {
        let path = IgnorePath::parse_object_fields(input, span)?;
        Ok(IgnorePaths {
            span,
            paths: vec![path],
        })
    }

    fn parse_list(input: ParseStream, span: Span) -> syn::Result<Self> {
        let ignore_paths = Punctuated::<IgnorePath, Token![,]>::parse_terminated(input)?;
        Ok(IgnorePaths {
            span,
            paths: ignore_paths.into_iter().collect(),
        })
    }

    fn parse_literal(lit: proc_macro2::Literal) -> syn::Result<Self> {
        let tokens = lit.to_token_stream();
        let lit_str: LitStr = syn::parse2(tokens)?;
        Ok(Self::from_lit_str(lit_str))
    }

    fn from_lit_str(lit_str: LitStr) -> Self {
        let span = lit_str.span();
        let paths = vec![IgnorePath::from_lit_str(lit_str)];
        IgnorePaths { paths, span }
    }
}

pub struct IgnorePath {
    span: Span,
    path: LitStr,
    reason: Option<LitStr>,
}

impl Parse for IgnorePath {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.step(|cursor| match cursor.token_tree() {
            Some((proc_macro2::TokenTree::Group(group), next)) => {
                Ok((Self::parse_group(group)?, next))
            }
            Some((proc_macro2::TokenTree::Literal(lit), next)) => {
                Ok((Self::parse_literal(lit)?, next))
            }
            _ => Err(syn::Error::new(
                input.span(),
                "Expected a string literal or object",
            )),
        })
    }
}

impl Spanned for IgnorePath {
    fn span(&self) -> Span {
        self.span
    }
}

impl IgnorePath {
    pub(crate) fn path(&self) -> &LitStr {
        &self.path
    }

    pub(crate) fn reason(&self) -> &Option<LitStr> {
        &self.reason
    }

    fn from_lit_str(lit_str: LitStr) -> Self {
        let span = lit_str.span();
        IgnorePath {
            span,
            path: lit_str,
            reason: None,
        }
    }

    fn parse_group(group: proc_macro2::Group) -> syn::Result<Self> {
        match group.delimiter() {
            proc_macro2::Delimiter::Brace => syn::parse::Parser::parse2(
                |input: ParseStream| IgnorePath::parse_object_fields(input, group.span()),
                group.stream(),
            ),
            _ => Err(syn::Error::new(
                group.span(),
                "Expected a string literal or object",
            )),
        }
    }

    fn parse_object_fields(input: ParseStream, span: Span) -> syn::Result<Self> {
        let mut path = None;
        let mut reason = None;

        while !input.is_empty() {
            let assignment = input.parse::<Assignment<LitStr>>()?;
            match assignment.ident().to_string().as_str() {
                "path" => {
                    if path.is_some() {
                        return Err(syn::Error::new(
                            assignment.ident().span(),
                            "Duplicate assignment.",
                        ));
                    }
                    path = Some(assignment.into_value());
                }
                "reason" => {
                    if reason.is_some() {
                        return Err(syn::Error::new(
                            assignment.ident().span(),
                            "Duplicate assignment.",
                        ));
                    }
                    reason = Some(assignment.into_value());
                }
                _ => {
                    return Err(syn::Error::new(
                        assignment.ident().span(),
                        "Invalid field identifier. Expected 'path' or 'reason'.",
                    ))
                }
            }
            if input.is_empty() {
                break;
            }
            input.parse::<Token![,]>()?;
        }

        Ok(IgnorePath {
            span,
            path: path.ok_or_else(|| syn::Error::new(span, "The 'path' field is missing."))?,
            reason,
        })
    }

    fn parse_literal(lit: proc_macro2::Literal) -> syn::Result<Self> {
        let tokens = lit.to_token_stream();
        let lit_str: LitStr = syn::parse2(tokens)?;
        Ok(IgnorePath {
            span: lit_str.span(),
            path: lit_str,
            reason: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::IgnoreConfig;

    #[test]
    fn correctly_parses_basic_ignore_config() {
        let input = r#"
            ["fixtures/ignored"]
        "#;
        let config = syn::parse_str::<IgnoreConfig>(input).expect("Failed to parse ignore config");

        assert_eq!(config.paths.paths.len(), 1);
        assert_eq!(config.paths.paths[0].path().value(), "fixtures/ignored");
        assert!(config.paths.paths[0].reason().is_none());
        assert!(config.reason.is_none());
    }

    #[test]
    fn correctly_parses_basic_object_ignore_config() {
        let input = r#"
            {
                paths = ["fixtures/ignored"],
            }
        "#;
        let config = syn::parse_str::<IgnoreConfig>(input).expect("Failed to parse ignore config");

        assert_eq!(config.paths.paths.len(), 1);
        assert_eq!(config.paths.paths[0].path().value(), "fixtures/ignored");
        assert!(config.paths.paths[0].reason().is_none());
        assert!(config.reason.is_none());
    }

    #[test]
    fn correctly_parses_object_ignore_config_with_default_reason() {
        let input = r#"
            {
                paths = ["fixtures/ignored"],
                reason = "some good reason"
            }
        "#;
        let config = syn::parse_str::<IgnoreConfig>(input).expect("Failed to parse ignore config");

        assert_eq!(config.paths.paths.len(), 1);
        assert_eq!(config.paths.paths[0].path().value(), "fixtures/ignored");
        assert!(config.paths.paths[0].reason().is_none());
        assert!(config.reason.is_some());
        assert_eq!(config.reason.unwrap().value(), "some good reason");
    }

    #[test]
    fn correctly_parses_object_ignore_config_with_basic_object_path() {
        let input = r#"
            {
                paths = {
                    path = "fixtures/ignored",
                },
            }
        "#;
        let config = syn::parse_str::<IgnoreConfig>(input).expect("Failed to parse ignore config");

        assert_eq!(config.paths.paths.len(), 1);
        assert_eq!(config.paths.paths[0].path().value(), "fixtures/ignored");
        assert!(config.paths.paths[0].reason().is_none());
        assert!(config.reason.is_none());
    }

    #[test]
    fn correctly_parses_object_ignore_config_with_basic_object_path_with_reason() {
        let input = r#"
            {
                paths = {
                    path = "fixtures/ignored",
                    reason = "some good reason",
                },
            }
        "#;
        let config = syn::parse_str::<IgnoreConfig>(input).expect("Failed to parse ignore config");

        assert_eq!(config.paths.paths.len(), 1);
        assert_eq!(config.paths.paths[0].path().value(), "fixtures/ignored");
        assert!(config.paths.paths[0].reason().is_some());
        assert_eq!(
            config.paths.paths[0].reason().as_ref().unwrap().value(),
            "some good reason"
        );
        assert!(config.reason.is_none());
    }

    #[test]
    fn correctly_parses_object_ignore_config_with_basic_object_paths() {
        let input = r#"
            {
                paths = [{
                    path = "fixtures/ignored",
                }],
            }
        "#;
        let config = syn::parse_str::<IgnoreConfig>(input).expect("Failed to parse ignore config");

        assert_eq!(config.paths.paths.len(), 1);
        assert_eq!(config.paths.paths[0].path().value(), "fixtures/ignored");
        assert!(config.paths.paths[0].reason().is_none());
        assert!(config.reason.is_none());
    }

    #[test]
    fn correctly_parses_object_ignore_config_with_basic_object_paths_with_reason() {
        let input = r#"
            {
                paths = [{
                    path = "fixtures/ignored",
                    reason = "some good reason",
                }],
            }
        "#;
        let config = syn::parse_str::<IgnoreConfig>(input).expect("Failed to parse ignore config");

        assert_eq!(config.paths.paths.len(), 1);
        assert_eq!(config.paths.paths[0].path().value(), "fixtures/ignored");
        assert!(config.paths.paths[0].reason().is_some());
        assert_eq!(
            config.paths.paths[0].reason().as_ref().unwrap().value(),
            "some good reason"
        );
        assert!(config.reason.is_none());
    }

    #[test]
    fn correctly_parses_object_ignore_config_with_mixed_paths() {
        let input = r#"
            {
                paths = [
                    "fixtures/ignored.0",
                    {
                        path = "fixtures/ignored.1",
                        reason = "some good reason",
                    },
                    {
                        path = "fixtures/ignored.2",
                    },
                ],
                reason = "some default reason",
            }
        "#;
        let config = syn::parse_str::<IgnoreConfig>(input).expect("Failed to parse ignore config");

        assert_eq!(config.paths.paths.len(), 3);
        assert_eq!(config.paths.paths[0].path().value(), "fixtures/ignored.0");
        assert!(config.paths.paths[0].reason().is_none());
        assert_eq!(config.paths.paths[1].path().value(), "fixtures/ignored.1");
        assert!(config.paths.paths[1].reason().is_some());
        assert_eq!(
            config.paths.paths[1].reason().as_ref().unwrap().value(),
            "some good reason"
        );
        assert_eq!(config.paths.paths[2].path().value(), "fixtures/ignored.2");
        assert!(config.paths.paths[2].reason().is_none());
        assert!(config.reason.is_some());
        assert_eq!(config.reason.unwrap().value(), "some default reason");
    }

    #[test]
    fn correctly_parses_object_ignore_paths_list() {
        let input = r#"
            [{
                path = "fixtures/ignored",
            }]
        "#;
        let config = syn::parse_str::<IgnoreConfig>(input).expect("Failed to parse ignore config");

        assert_eq!(config.paths.paths.len(), 1);
        assert_eq!(config.paths.paths[0].path().value(), "fixtures/ignored");
        assert!(config.paths.paths[0].reason().is_none());
        assert!(config.reason.is_none());
    }

    #[test]
    fn correctly_parses_object_ignore_paths_list_with_reason() {
        let input = r#"
            [{
                path = "fixtures/ignored",
                reason = "some good reason",
            }]
        "#;
        let config = syn::parse_str::<IgnoreConfig>(input).expect("Failed to parse ignore config");

        assert_eq!(config.paths.paths.len(), 1);
        assert_eq!(config.paths.paths[0].path().value(), "fixtures/ignored");
        assert!(config.paths.paths[0].reason().is_some());
        assert_eq!(
            config.paths.paths[0].reason().as_ref().unwrap().value(),
            "some good reason"
        );
        assert!(config.reason.is_none());
    }
}
