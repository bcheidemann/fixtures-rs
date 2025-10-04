use proc_macro2::Span;
use syn::{AttrStyle, Attribute, MacroDelimiter, Meta};

use crate::{parse::spanned::Spanned, utils::attribute::attribute_path_is};

use super::ignore_args::IgnoreArgs;

pub struct IgnoreAttribute {
    span: Span,
    pub args: IgnoreArgs,
}

impl IgnoreAttribute {
    pub fn try_from_attribute(attr: &Attribute) -> syn::Result<Option<Self>> {
        if attr.style != AttrStyle::Outer {
            return Ok(None);
        }

        let Meta::List(meta) = &attr.meta else {
            return Ok(None);
        };

        if !attribute_path_is(&meta.path, ["fixtures", "ignore"]) {
            return Ok(None);
        }

        if !matches!(meta.delimiter, MacroDelimiter::Paren(_)) {
            return Err(syn::Error::new(
                meta.delimiter.span().span(),
                "Expected parentheses",
            ));
        }

        Ok(Some(Self {
            span: attr.span(),
            args: syn::parse2::<IgnoreArgs>(meta.tokens.clone())?,
        }))
    }
}

impl Spanned for IgnoreAttribute {
    fn span(&self) -> Span {
        self.span
    }
}
