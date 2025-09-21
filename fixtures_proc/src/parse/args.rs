use syn::{
    ext::IdentExt as _,
    parse::{Parse, ParseStream},
    Ident, Token,
};

use super::{
    ignore_config::IgnoreConfig, option_assignment::OptionAssignment, paths::Paths,
    spanned::Spanned as _,
};

pub struct Args {
    include: Paths,
    ignore: Option<IgnoreConfig>,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let include: Paths = input.parse()?;
        input.parse::<Option<Token![,]>>()?;

        let mut ignore = None;

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
                        ignore = Some(ignore_option_assignment.into_ignore_config());
                    }
                }
            }
            if input.is_empty() {
                break;
            }
            input.parse::<Token![,]>()?;
        }
        Ok(Args { include, ignore })
    }
}

impl Args {
    pub fn include(&self) -> &Paths {
        &self.include
    }

    pub fn ignore(&self) -> &Option<IgnoreConfig> {
        &self.ignore
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
        assert!(args.ignore.unwrap().paths().paths().is_empty());
    }

    #[test]
    fn correctly_parses_empty_ignore_with_trailing_comma() {
        let input = r#"["fixtures/*.txt"], ignore = [],"#;
        let args: Args = syn::parse_str(input).expect("Failed to parse args");

        assert!(args.ignore.is_some());
        assert!(args.ignore.unwrap().paths().paths().is_empty());
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
        assert_eq!(ignore.paths().paths().len(), 1);
        assert_eq!(
            ignore.paths().paths()[0].path().value(),
            "fixtures/*.ignore.txt"
        );
    }

    #[test]
    fn correctly_parses_complex_ignore() {
        let input = r#"
            ["fixtures/*.txt"],
            ignore = {
                paths = ["fixtures/*.ignore.txt"],
                reason = "reason for ignoring file",
            },
        "#;
        let args: Args = syn::parse_str(input).expect("Failed to parse args");

        assert!(args.ignore.is_some());
        assert_eq!(args.ignore.as_ref().unwrap().paths().paths().len(), 1);
        assert_eq!(
            args.ignore.as_ref().unwrap().paths().paths()[0]
                .path()
                .value(),
            "fixtures/*.ignore.txt"
        );
        assert!(args.ignore.as_ref().unwrap().reason().is_some());
        assert_eq!(
            args.ignore
                .as_ref()
                .unwrap()
                .reason()
                .as_ref()
                .unwrap()
                .value(),
            "reason for ignoring file",
        );
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
}
