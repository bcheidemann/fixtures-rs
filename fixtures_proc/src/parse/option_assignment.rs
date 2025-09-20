use proc_macro2::Span;
use syn::{
    parse::{Parse, ParseStream},
    Ident, LitStr, Token,
};

use super::paths::Paths;

pub enum OptionAssignment {
    Ignore(IgnoreOptionAssignment),
    IgnoreReason(IgnoreReasonOptionAssignment),
}

impl Parse for OptionAssignment {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if let Some(ident) = input.parse::<Option<Ident>>()? {
            let ident_name = ident.to_string();
            return match ident_name.as_str() {
                "ignore" => Ok(OptionAssignment::Ignore(
                    IgnoreOptionAssignment::parse_from_ident(input, ident)?,
                )),
                "ignore_reason" => Ok(OptionAssignment::IgnoreReason(
                    IgnoreReasonOptionAssignment::parse_from_ident(input, ident)?,
                )),
                _ => Err(syn::Error::new(ident.span(), "Invalid option")),
            };
        }
        Err(syn::Error::new(input.span(), "Expected option identifier"))
    }
}

pub struct IgnoreOptionAssignment {
    span: Span,
    paths: Paths,
}

impl IgnoreOptionAssignment {
    pub fn span(&self) -> Span {
        self.span
    }

    pub fn into_paths(self) -> Paths {
        self.paths
    }

    fn parse_from_ident(input: ParseStream, ident: Ident) -> syn::Result<Self> {
        input.parse::<Token![=]>()?;
        let paths: Paths = input.parse()?;
        let span = ident
            .span()
            .join(paths.span())
            // On non-nightly compilers, join will always return None
            .unwrap_or_else(|| paths.span());
        Ok(IgnoreOptionAssignment { span, paths })
    }
}

pub struct IgnoreReasonOptionAssignment {
    span: Span,
    reason: LitStr,
}

impl IgnoreReasonOptionAssignment {
    pub fn span(&self) -> Span {
        self.span
    }

    pub fn into_lit_str(self) -> LitStr {
        self.reason
    }

    fn parse_from_ident(input: ParseStream, ident: Ident) -> syn::Result<Self> {
        input.parse::<Token![=]>()?;
        let lit_str: LitStr = input.parse()?;
        let span = ident
            .span()
            .join(lit_str.span())
            // On non-nightly compilers, join will always return None
            .unwrap_or_else(|| lit_str.span());
        Ok(IgnoreReasonOptionAssignment {
            span,
            reason: lit_str,
        })
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
        let OptionAssignment::Ignore(ignore_option_assignment) = option_assignment else {
            unreachable!();
        };
        assert!(ignore_option_assignment.paths.paths().is_empty());
    }

    #[test]
    fn correctly_parses_ignore_option_assignment_with_one_path() {
        let input = r#"ignore = ["fixtures/*.ignore.txt"]"#;
        let option_assignment: OptionAssignment =
            syn::parse_str(input).expect("Failed to parse option assignment");

        assert!(matches!(option_assignment, OptionAssignment::Ignore(_)));
        let OptionAssignment::Ignore(ignore_option_assignment) = option_assignment else {
            unreachable!();
        };
        assert_eq!(ignore_option_assignment.paths.paths().len(), 1);
        assert_eq!(
            ignore_option_assignment.paths.paths()[0].value(),
            "fixtures/*.ignore.txt"
        );
    }

    #[test]
    fn correctly_parses_ignore_reason_option_assignment() {
        let input = r#"ignore_reason = "the provided reason""#;
        let option_assignment: OptionAssignment =
            syn::parse_str(input).expect("Failed to parse option assignment");

        assert!(matches!(
            option_assignment,
            OptionAssignment::IgnoreReason(_)
        ));
        let OptionAssignment::IgnoreReason(ignore_reason_option_assignment) = option_assignment
        else {
            unreachable!();
        };
        assert_eq!(
            ignore_reason_option_assignment.reason.value(),
            "the provided reason"
        );
    }
}
