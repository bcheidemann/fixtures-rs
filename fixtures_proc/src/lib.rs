extern crate proc_macro;

mod ignore_matcher;
mod parse;

use ignore_matcher::{IgnoreMatcher, MatchResult};
use parse::spanned::Spanned;
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, parse_quote, punctuated::Punctuated, AttrStyle, FnArg, Ident, ItemFn,
    LitStr, Meta, Pat, Path, Token,
};

struct TestFnExpansion {
    ident: Ident,
    tokens: proc_macro2::TokenStream,
}

#[proc_macro_attribute]
pub fn fixtures(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as parse::args::Args);

    let current_dir = std::env::current_dir().expect("failed to get current directory");
    let paths_iterator = globwalk::GlobWalkerBuilder::from_patterns(
        &current_dir,
        &args
            .include()
            .paths()
            .iter()
            .map(|lit_glob_path| lit_glob_path.value())
            .collect::<Vec<_>>(),
    )
    .build()
    .expect("failed to build glob walker")
    .filter_map(Result::ok);

    let ignore_matcher = if let Some(config) = args.ignore() {
        match IgnoreMatcher::new(config) {
            Ok(matcher) => matcher,
            Err((path, err)) => {
                return syn::Error::new(path.span(), format!("{err}"))
                    .to_compile_error()
                    .into()
            }
        }
    } else {
        IgnoreMatcher::empty()
    };

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

    if let Some(ignore) = args.ignore() {
        if !is_test {
            return syn::Error::new(ignore.span(), "The ignore option is only valid for test functions. This function doesn't have a `#[test]` attribute.")
                .to_compile_error()
                .into();
        }
    }

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
                args.include().span(),
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
            let maybe_ignore_attr = match ignore_matcher.matched(path.path()) {
                MatchResult::Matched {
                    reason: Some(reason),
                } => parse_quote!(#[ignore = #reason]),
                MatchResult::Matched { reason: None } => parse_quote!(#[ignore]),
                MatchResult::Unmatched => proc_macro2::TokenStream::new(),
            };
            let tokens = quote! {
                #(#fn_attrs)*
                #maybe_ignore_attr
                pub fn #ident(#fn_non_path_args) #fn_output {
                    #fn_name(::std::path::Path::new(#lit_file_path), #fn_non_path_args_idents)
                }
            };
            Some(TestFnExpansion { ident, tokens })
        })
        .collect::<Vec<_>>();

    if expansions.is_empty() {
        return syn::Error::new(args.include().span(), "No valid files found".to_string())
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
        #maybe_cfg_test_attr
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
