use proc_macro2::Span;
use syn::{
    parse::{Parse, ParseStream},
    Ident, Token,
};

use super::{legacy_ignore_config::LegacyIgnoreConfig, spanned::Spanned};

pub enum OptionAssignment {
    Ignore(IgnoreOptionAssignment),
}

impl Parse for OptionAssignment {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if let Some(ident) = input.parse::<Option<Ident>>()? {
            let ident_name = ident.to_string();
            return match ident_name.as_str() {
                "ignore" => Ok(OptionAssignment::Ignore(
                    IgnoreOptionAssignment::parse_from_ident(input, ident)?,
                )),
                _ => Err(syn::Error::new(ident.span(), "Invalid option")),
            };
        }
        Err(syn::Error::new(input.span(), "Expected option identifier"))
    }
}

pub struct IgnoreOptionAssignment {
    span: Span,
    config: LegacyIgnoreConfig,
}

impl IgnoreOptionAssignment {
    pub fn into_ignore_config(self) -> LegacyIgnoreConfig {
        self.config
    }

    fn parse_from_ident(input: ParseStream, ident: Ident) -> syn::Result<Self> {
        input.parse::<Token![=]>()?;
        let config: LegacyIgnoreConfig = input.parse()?;
        let span = ident
            .span()
            .join(config.span())
            // On non-nightly compilers, join will always return None
            .unwrap_or_else(|| config.span());
        Ok(IgnoreOptionAssignment { span, config })
    }
}

impl Spanned for IgnoreOptionAssignment {
    fn span(&self) -> Span {
        self.span
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn correctly_parses_ignore_option_assignment_without_paths() {
        let input = r#"ignore = []"#;
        let option_assignment: OptionAssignment =
            syn::parse_str(input).expect("Failed to parse option assignment");

        assert!(matches!(option_assignment, OptionAssignment::Ignore(_)));
        let OptionAssignment::Ignore(ignore_option_assignment) = option_assignment;
        assert!(ignore_option_assignment.config.paths().paths().is_empty());
    }

    #[test]
    fn correctly_parses_ignore_option_assignment_with_one_path() {
        let input = r#"ignore = ["fixtures/*.ignore.txt"]"#;
        let option_assignment: OptionAssignment =
            syn::parse_str(input).expect("Failed to parse option assignment");

        assert!(matches!(option_assignment, OptionAssignment::Ignore(_)));
        let OptionAssignment::Ignore(ignore_option_assignment) = option_assignment;
        assert_eq!(ignore_option_assignment.config.paths().paths().len(), 1);
        assert_eq!(
            ignore_option_assignment.config.paths().paths()[0]
                .path()
                .value(),
            "fixtures/*.ignore.txt"
        );
    }
}
