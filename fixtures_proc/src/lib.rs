extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, spanned::Spanned as _, Expr, ExprArray, Ident, ItemFn, Lit, LitStr};

#[proc_macro_attribute]
pub fn fixtures(args: TokenStream, input: TokenStream) -> TokenStream {
    let patterns = parse_macro_input!(args as ExprArray);

    let mut glob_paths = Vec::with_capacity(patterns.elems.len());

    for glob_elem in &patterns.elems {
        if let Expr::Lit(glob_lit) = glob_elem {
            if let Lit::Str(ref glob_path) = glob_lit.lit {
                glob_paths.push(glob_path);
            } else {
                return syn::Error::new(glob_lit.span(), "Expected a string literal")
                    .to_compile_error()
                    .into();
            };
        } else {
            return syn::Error::new(glob_elem.span(), "Expected a string literal")
                .to_compile_error()
                .into();
        }
    }

    let paths_iterator = globwalk::GlobWalkerBuilder::from_patterns(
        std::env::current_dir().expect("failed to get current directory"),
        &glob_paths
            .iter()
            .map(|glob_path| glob_path.value())
            .collect::<Vec<_>>(),
    )
    .build()
    .expect("failed to build glob walker")
    .filter_map(Result::ok);

    let test_fn = parse_macro_input!(input as ItemFn);

    let fn_attrs = &test_fn.attrs;
    let fn_name = &test_fn.sig.ident;
    let fn_args = &test_fn.sig.inputs;
    let fn_block = &test_fn.block;

    let mut file_names = std::collections::HashMap::new();

    let expanded = paths_iterator
        .filter_map(|path| {
            let file_name = path.file_name().to_str()?.to_owned();
            let fn_file_name = file_name
                .replace('.', "_dot_")
                .replace(|c: char| !c.is_ascii_alphanumeric(), "_");
            let lit_file_path = LitStr::new(
                path.path()
                    .to_str()
                    .expect("file path should be valid UTF-8"),
                patterns.span(),
            );
            let similar_file_names = file_names.entry(file_name.clone()).or_insert(0usize);
            *similar_file_names += 1;
            let lit_test_name = if *similar_file_names == 1 {
                Ident::new(&format!("{fn_name}_{fn_file_name}"), fn_name.span())
            } else {
                Ident::new(
                    &format!("{fn_name}_{fn_file_name}_{similar_file_names}"),
                    fn_name.span(),
                )
            };

            Some(quote! {
                #(#fn_attrs)*
                fn #lit_test_name() {
                    #fn_name(::std::path::Path::new(#lit_file_path));
                }
            })
        })
        .collect::<Vec<_>>();

    if expanded.is_empty() {
        return syn::Error::new(
            patterns.span(),
            format!("No valid files found for glob pattern: {glob_paths:?}"),
        )
        .into_compile_error()
        .into();
    }

    quote! {
        fn #fn_name(#fn_args) #fn_block
        #(#expanded)*
    }
    .into()
}
