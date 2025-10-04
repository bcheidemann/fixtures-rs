use syn::{
    parse::{Parse, ParseStream},
    Ident, LitStr, Token,
};

use super::assignment::Assignment;

pub struct IgnoreArgs {
    pub paths: LitStr,
    pub reason: Option<LitStr>,
}

impl Parse for IgnoreArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(Ident) {
            Self::parse_fields(input)
        } else {
            Self::parse_literal(input)
        }
    }
}

impl IgnoreArgs {
    fn parse_literal(input: ParseStream) -> syn::Result<Self> {
        let path = input.parse::<LitStr>()?;
        if !input.is_empty() {
            return Err(syn::Error::new(input.span(), "Unexpected token."));
        }
        Ok(Self {
            paths: path,
            reason: None,
        })
    }

    fn parse_fields(input: ParseStream) -> syn::Result<Self> {
        let error_span = input.span();
        let mut paths = None;
        let mut reason = None;

        while !input.is_empty() {
            let assignment = input.parse::<Assignment<LitStr>>()?;

            match assignment.ident().to_string().as_str() {
                "paths" => {
                    if paths.is_some() {
                        return Err(syn::Error::new(
                            assignment.ident().span(),
                            "Duplicate assignment.",
                        ));
                    }
                    paths = Some(assignment.into_value());
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
                        "Invalid field identifier. Expected 'paths' or 'reason'.",
                    ))
                }
            }

            if input.is_empty() {
                break;
            }

            input.parse::<Token![,]>()?;
        }

        Ok(Self {
            paths: paths
                .ok_or_else(|| syn::Error::new(error_span, "The 'path' field is missing."))?,
            reason,
        })
    }
}
