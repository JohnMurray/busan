extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Message)]
pub fn message(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;

    let expanded = quote! {
        impl ::busan::message::Message for #name {
            fn as_any(&self) -> &dyn ::std::any::Any {
                self
            }
        }
    };

    proc_macro::TokenStream::from(expanded)
}
