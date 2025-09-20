use syn::{
    ext::IdentExt as _,
    parse::{Parse, ParseStream},
    Ident, LitStr, Token,
};

use super::{option_assignment::OptionAssignment, paths::Paths};

pub struct Args {
    include: Paths,
    ignore: Option<Paths>,
    ignore_reason: Option<LitStr>,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let include: Paths = input.parse()?;
        input.parse::<Option<Token![,]>>()?;

        let mut ignore = None;
        let mut ignore_reason = None;

        while !input.is_empty() {
            if input.peek(Ident::peek_any) {
                match input.parse::<OptionAssignment>()? {
                    OptionAssignment::Ignore(ignore_option_assignment) => {
                        if ignore.is_some() {
                            return Err(syn::Error::new(
                                ignore_option_assignment.span(),
                                "Duplicate ignore assignment",
                            ));
                        }
                        ignore = Some(ignore_option_assignment.into_paths());
                    }
                    OptionAssignment::IgnoreReason(ignore_reason_option_assignment) => {
                        if ignore_reason.is_some() {
                            return Err(syn::Error::new(
                                ignore_reason_option_assignment.span(),
                                "Duplicate ignore_reason assignment",
                            ));
                        }
                        ignore_reason = Some(ignore_reason_option_assignment.into_lit_str());
                    }
                }
            }
            input.parse::<Option<Token![,]>>()?;
        }
        Ok(Args {
            include,
            ignore,
            ignore_reason,
        })
    }
}

impl Args {
    pub fn include(&self) -> &Paths {
        &self.include
    }

    pub fn ignore(&self) -> &Option<Paths> {
        &self.ignore
    }

    pub fn ignore_reason(&self) -> &Option<LitStr> {
        &self.ignore_reason
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn correctly_parses_anonymous_include() {
        let input = r#"["fixtures/*.txt"]"#;
        let args: Args = syn::parse_str(input).expect("Failed to parse args");

        assert_eq!(args.include.paths().len(), 1);
        assert_eq!(args.include.paths()[0].value(), "fixtures/*.txt");
        assert!(args.ignore.is_none());
    }

    #[test]
    fn correctly_parses_anonymous_include_with_trailing_comma() {
        let input = r#"["fixtures/*.txt"],"#;
        let args: Args = syn::parse_str(input).expect("Failed to parse args");

        assert_eq!(args.include.paths().len(), 1);
        assert_eq!(args.include.paths()[0].value(), "fixtures/*.txt");
        assert!(args.ignore.is_none());
    }

    #[test]
    fn correctly_parses_empty_ignore() {
        let input = r#"["fixtures/*.txt"], ignore = []"#;
        let args: Args = syn::parse_str(input).expect("Failed to parse args");

        assert!(args.ignore.is_some());
        assert!(args.ignore.unwrap().paths().is_empty());
    }

    #[test]
    fn correctly_parses_empty_ignore_with_trailing_comma() {
        let input = r#"["fixtures/*.txt"], ignore = [],"#;
        let args: Args = syn::parse_str(input).expect("Failed to parse args");

        assert!(args.ignore.is_some());
        assert!(args.ignore.unwrap().paths().is_empty());
    }

    #[test]
    fn correctly_parses_ignore_with_paths() {
        let input = r#"
            ["fixtures/*.txt"],
            ignore = ["fixtures/*.ignore.txt"]
        "#;
        let args: Args = syn::parse_str(input).expect("Failed to parse args");

        assert!(args.ignore.is_some());
        let ignore = args.ignore.unwrap();
        assert_eq!(ignore.paths().len(), 1);
        assert_eq!(ignore.paths()[0].value(), "fixtures/*.ignore.txt");
    }

    #[test]
    fn returns_error_on_duplicate_ignore_assignments() {
        let input = r#"
            ["fixtures/*.txt"],
            ignore = [],
            ignore = [],
        "#;
        let result = syn::parse_str::<Args>(input);

        assert!(result.is_err());
    }

    #[test]
    fn correctly_parses_ignore_with_reason() {
        let input = r#"
            ["fixtures/*.txt"],
            ignore = [],
            ignore_reason = "the provided reason",
        "#;
        let args: Args = syn::parse_str(input).expect("Failed to parse args");

        assert!(args.ignore_reason.is_some());
        assert_eq!(args.ignore_reason.unwrap().value(), "the provided reason");
    }

    #[test]
    fn returns_error_on_duplicate_ignore_reason_assignments() {
        let input = r#"
            ["fixtures/*.txt"],
            ignore = []
            ignore_reason = "",
            ignore_reason = "",
        "#;
        let result = syn::parse_str::<Args>(input);

        assert!(result.is_err());
    }
}
