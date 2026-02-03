use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

#[proc_macro]
pub fn generate_tests(input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as syn::ExprTuple);
    let mut args_iter = args.elems.iter();

    let func = args_iter.next().expect("Missing function path").clone();
    let dir = args_iter.next().expect("Missing directory path").clone();
    let ext = args_iter.next().expect("Missing file extension").clone();
    let extra = args_iter.next().map(|v| match v {
        syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(lit),
            ..
        }) => lit.value(),
        _ => panic!("Expected directory path as a string literal"),
    });

    let dir_path = match dir {
        syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(lit),
            ..
        }) => lit.value(),
        _ => panic!("Expected directory path as a string literal"),
    };

    let extension = match ext {
        syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(lit),
            ..
        }) => lit.value(),
        _ => panic!("Expected file extension as a string literal"),
    };

    // Read the directory and filter files based on extension.
    let mut test_functions = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&dir_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some(&extension) {
                if let Some(file_stem) = path.file_stem().and_then(|s| s.to_str()) {
                    let test_fn_name = syn::Ident::new(
                        &format!(
                            "{}_{}{}",
                            file_stem.replace("-", "_"),
                            extension,
                            extra.as_ref().map(|v| format!("_{v}")).unwrap_or_default()
                        ),
                        proc_macro2::Span::call_site(),
                    );
                    let a = file_stem.to_owned();
                    let content = std::fs::canonicalize(path).unwrap().display().to_string();

                    test_functions.push(quote! {
                        #[tokio::test]
                        async fn #test_fn_name() {
                            #func(#a, &std::fs::read_to_string(#content).unwrap()).await;
                        }
                    });
                }
            }
        }
    } else {
        panic!("Failed to read directory");
    }

    // Generate the final output.
    let expanded = quote! {
        #(#test_functions)*
    };

    TokenStream::from(expanded)
}
