extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, parse_quote, punctuated::Punctuated, spanned::Spanned as _, AttrStyle, Expr,
    ExprArray, FnArg, Ident, ItemFn, Lit, LitStr, Meta, Pat, Path, Token,
};

struct TestFnExpansion {
    ident: Ident,
    tokens: proc_macro2::TokenStream,
}

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
    let fn_output = &test_fn.sig.output;
    let fn_block = &test_fn.block;

    let fn_non_path_args = {
        let mut remaining_args = Punctuated::<&FnArg, Token![,]>::new();
        for fn_arg in fn_args.iter().skip(1) {
            remaining_args.push(fn_arg);
        }
        remaining_args
    };
    let fn_non_path_args_idents = {
        let mut idents = Punctuated::<&Ident, Token![,]>::new();
        for arg in fn_non_path_args.iter() {
            if let FnArg::Typed(pat_ty) = arg {
                if let Pat::Ident(ident) = pat_ty.pat.as_ref() {
                    idents.push(&ident.ident);
                    continue;
                }
                return syn::Error::new(arg.span(), "Expected an identity, but found a pattern")
                    .to_compile_error()
                    .into();
            }
            return syn::Error::new(arg.span(), "Unexpected receiver argument")
                .to_compile_error()
                .into();
        }
        idents
    };

    let is_test = fn_attrs.iter().any(|attr| {
        if attr.style != AttrStyle::Outer {
            return false;
        }
        if let Meta::Path(Path {
            leading_colon: None,
            segments,
        }) = &attr.meta
        {
            if segments.len() != 1 {
                return false;
            }
            let path_segment = segments.first().unwrap();
            return path_segment.ident == "test";
        }
        false
    });

    let mut file_names = std::collections::HashMap::new();

    let expansions = paths_iterator
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
            let ident = if *similar_file_names == 1 {
                Ident::new(&fn_file_name, fn_name.span())
            } else {
                Ident::new(
                    &format!("{fn_file_name}_{similar_file_names}"),
                    fn_name.span(),
                )
            };
            let tokens = quote! {
                #(#fn_attrs)*
                pub fn #ident(#fn_non_path_args) #fn_output {
                    #fn_name(::std::path::Path::new(#lit_file_path), #fn_non_path_args_idents)
                }
            };
            Some(TestFnExpansion { ident, tokens })
        })
        .collect::<Vec<_>>();

    if expansions.is_empty() {
        return syn::Error::new(
            patterns.span(),
            format!("No valid files found for glob pattern: {glob_paths:?}"),
        )
        .into_compile_error()
        .into();
    }

    let fn_expansions = expansions.iter().map(|expansion| &expansion.tokens);
    let expansion_idents = {
        let mut impl_idents = Punctuated::<&Ident, Token![,]>::new();
        for expansion in expansions.iter() {
            impl_idents.push(&expansion.ident);
        }
        impl_idents
    };

    let maybe_cfg_test_attr = if is_test {
        parse_quote!(#[cfg(test)])
    } else {
        proc_macro2::TokenStream::new()
    };

    let output = quote! {
        fn #fn_name(#fn_args) #fn_output #fn_block
        #maybe_cfg_test_attr
        mod #fn_name {
            use super::*;

            #(#fn_expansions)*

            pub const EXPANSIONS: &[fn(#fn_non_path_args) #fn_output] = &[#expansion_idents];
        }
    };

    output.into()
}
