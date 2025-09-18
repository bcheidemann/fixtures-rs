extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, punctuated::Punctuated, spanned::Spanned as _, Expr, ExprArray, FnArg,
    Ident, ItemFn, Lit, LitStr, Pat, Token,
};

struct TestFnExpansion {
    impl_ident: Ident,
    impl_tokens: proc_macro2::TokenStream,
    wrapper_tokens: proc_macro2::TokenStream,
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
            let lit_impl_name = if *similar_file_names == 1 {
                Ident::new(&fn_file_name, fn_name.span())
            } else {
                Ident::new(
                    &format!("{fn_file_name}_{similar_file_names}"),
                    fn_name.span(),
                )
            };
            let lit_wrapper_name = if *similar_file_names == 1 {
                Ident::new(&format!("{fn_name}_{fn_file_name}"), fn_name.span())
            } else {
                Ident::new(
                    &format!("{fn_name}_{fn_file_name}_{similar_file_names}"),
                    fn_name.span(),
                )
            };

            let impl_tokens = quote! {
                pub fn #lit_impl_name(#fn_non_path_args) #fn_output {
                    #fn_name(::std::path::Path::new(#lit_file_path), #fn_non_path_args_idents)
                }
            };

            // EXPLANATION: This wraper function exists to avoid errors in top level #[test] functions, i.e. where there
            //              is no enclosing #[cfg(test)] module. This error arrises because we generate a module with
            //              references to each expansion. When building against cfg(not(test)), the #[test] expansion
            //              would be removed, but the generated module would remain. This results in the EXPANSIONS
            //              const holding references to a non-existant test function. This is avoided by adding a
            //              wrapper function, which has the attributes from the original function, and calls an "impl"
            //              function without the attributes. When building against cfg(not(test)), the wrapper function
            //              is removed, but the "impl" function remains. Therefore, the EXPANSIONS const can safely hold
            //              references to the "impl" functions. The downside of this is that any attributes applied
            //              after #[fixtures(...)] are not applied to the "impl" function. In future, we might consider
            //              fixing this issue by checking for the presence of a #[test] or #[cfg(test)] attribute and
            //              instead applying a matching #[cfg(test)] attribute to the generated module.
            let wrapper_tokens = quote! {
                #(#fn_attrs)*
                fn #lit_wrapper_name(#fn_non_path_args) #fn_output {
                    #fn_name::#lit_impl_name(#fn_non_path_args_idents)
                }
            };

            Some(TestFnExpansion {
                impl_ident: lit_impl_name,
                impl_tokens,
                wrapper_tokens,
            })
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

    let expanded_fns_impl_tokens = expansions.iter().map(|expansion| &expansion.impl_tokens);
    let expanded_fns_wrapper_tokens = expansions.iter().map(|expansion| &expansion.wrapper_tokens);
    let expanded_impl_idents = {
        let mut impl_idents = Punctuated::<&Ident, Token![,]>::new();
        for expansion in expansions.iter() {
            impl_idents.push(&expansion.impl_ident);
        }
        impl_idents
    };

    let output = quote! {
        fn #fn_name(#fn_args) #fn_output #fn_block
        #(#expanded_fns_wrapper_tokens)*
        mod #fn_name {
            use super::*;

            #(#expanded_fns_impl_tokens)*

            pub const EXPANSIONS: &[fn(#fn_non_path_args) #fn_output] = &[#expanded_impl_idents];
        }
    };

    output.into()
}
