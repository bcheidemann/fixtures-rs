use proc_macro2::Span;
use syn::{
    parse::{Parse, ParseStream},
    Ident, Token,
};

use super::spanned::Spanned;

pub(crate) struct Assignment<TVal: Parse + Spanned> {
    span: Span,
    ident: Ident,
    value: TVal,
}

impl<TVal: Parse + Spanned> Parse for Assignment<TVal> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident = input.parse::<Ident>()?;
        input.parse::<Token![=]>()?;
        let value = input.parse::<TVal>()?;
        let span = ident
            .span()
            .join(value.span())
            // On non-nightly compilers, join will always return None
            .unwrap_or_else(|| value.span());
        Ok(Assignment { span, ident, value })
    }
}

impl<TVal: Parse + Spanned> Spanned for Assignment<TVal> {
    fn span(&self) -> Span {
        self.span
    }
}

impl<TVal: Parse + Spanned> Assignment<TVal> {
    pub fn ident(&self) -> &Ident {
        &self.ident
    }

    pub fn into_value(self) -> TVal {
        self.value
    }
}
