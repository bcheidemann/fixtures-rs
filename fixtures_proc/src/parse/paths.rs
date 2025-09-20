use proc_macro2::Span;
use syn::{
    parse::{Parse, ParseStream},
    spanned::Spanned,
    Expr, ExprArray, Lit, LitStr,
};

pub struct Paths {
    span: Span,
    paths: Vec<LitStr>,
}

impl Parse for Paths {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let expr_array = ExprArray::parse(input)?;
        let span = expr_array.span();

        let mut paths = Vec::with_capacity(expr_array.elems.len());

        for elem in expr_array.elems {
            if let Expr::Lit(expr_lit) = elem {
                if let Lit::Str(lit_str) = expr_lit.lit {
                    paths.push(lit_str);
                } else {
                    return Err(syn::Error::new(
                        expr_lit.span(),
                        "Expected a string literal",
                    ));
                };
            } else {
                return Err(syn::Error::new(elem.span(), "Expected a string literal"));
            }
        }

        Ok(Paths { span, paths })
    }
}

impl Paths {
    pub fn span(&self) -> Span {
        self.span
    }

    pub fn paths(&self) -> &Vec<LitStr> {
        &self.paths
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn correctly_parses_zero_paths() {
        let input = r#"[]"#;
        let paths: Paths = syn::parse_str(input).expect("Failed to parse paths");

        assert!(paths.paths.is_empty());
    }

    #[test]
    fn correctly_parses_one_path() {
        let input = r#"["fixtures/*.txt"]"#;
        let paths: Paths = syn::parse_str(input).expect("Failed to parse paths");

        assert_eq!(paths.paths.len(), 1);
        assert_eq!(paths.paths[0].value(), "fixtures/*.txt");
    }

    #[test]
    fn correctly_parses_multiple_path() {
        let input = r#"["fixtures/*.txt", "!fixtures/*.skip.txt"]"#;
        let paths: Paths = syn::parse_str(input).expect("Failed to parse paths");

        assert_eq!(paths.paths.len(), 2);
        assert_eq!(paths.paths[0].value(), "fixtures/*.txt");
        assert_eq!(paths.paths[1].value(), "!fixtures/*.skip.txt");
    }
}
