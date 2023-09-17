extern crate proc_macro;
use glob::glob;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Ident, ItemFn, Lit, LitStr};

#[proc_macro_attribute]
pub fn fixtures(args: TokenStream, input: TokenStream) -> TokenStream {
    let glob_lit = parse_macro_input!(args as Lit);
    let glob_path = if let Lit::Str(ref glob_path) = glob_lit {
        glob_path
    } else {
        return syn::Error::new(glob_lit.span(), "Expected a string literal")
            .to_compile_error()
            .into();
    };
    let test_fn = parse_macro_input!(input as ItemFn);

    let fn_name = &test_fn.sig.ident;
    let fn_args = &test_fn.sig.inputs;
    let fn_block = &test_fn.block;

    let paths = match glob(glob_path.value().as_str()) {
        Err(err) => {
            return syn::Error::new(
                glob_lit.span(),
                format!("Failed to read glob pattern: {}", err),
            )
            .into_compile_error()
            .into();
        }
        Ok(paths) => paths,
    };

    let mut file_names = std::collections::HashMap::new();

    let expanded = paths
        .filter_map(Result::ok)
        .filter_map(|path| {
            let file_name = path
                .file_name()
                .expect("Failed to get file name")
                .to_str()?
                .to_owned()
                .replace('.', "_dot_")
                .replace(|c: char| !c.is_ascii_alphanumeric(), "_");
            let lit_file_path = LitStr::new(path.to_str()?, glob_path.span());
            let similar_file_names = file_names.entry(file_name.clone()).or_insert(0usize);
            *similar_file_names += 1;
            let lit_test_name = Ident::new(
                &format!("{fn_name}_{file_name}_{similar_file_names}"),
                fn_name.span(),
            );

            Some(quote! {
                #[test]
                fn #lit_test_name() {
                    #fn_name(::std::path::Path::new(#lit_file_path));
                }
            })
        })
        .collect::<Vec<_>>();

    if expanded.is_empty() {
        return syn::Error::new(
            glob_lit.span(),
            format!(
                "No valid files found for glob pattern: {}",
                glob_path.value()
            ),
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
