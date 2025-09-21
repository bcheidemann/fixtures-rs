use proc_macro2::Span;
use syn::spanned::Spanned as SynSpanned;

pub trait Spanned {
    fn span(&self) -> Span;
}

impl<T: SynSpanned> Spanned for T {
    fn span(&self) -> Span {
        self.span()
    }
}
