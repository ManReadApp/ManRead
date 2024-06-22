extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, LitStr};

#[proc_macro]
pub fn strip_prefix(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as LitStr);
    let value = input.value();

    if value.starts_with('/') {
        let value = &value[1..];
        quote! {
            #value
        }
        .into()
    } else {
        return quote! {
            compile_error!("literal should not start with '/'");
        }
        .into();
    }
}
