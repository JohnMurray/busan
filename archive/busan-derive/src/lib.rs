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

            fn encode_to_vec2(&self) -> Vec<u8> {
                prost::Message::encode_to_vec(self)
            }

            fn merge2(&mut self, bytes: &[u8]) -> Result<(), prost::DecodeError> {
                prost::Message::merge(self, bytes)
            }
        }
    };

    proc_macro::TokenStream::from(expanded)
}
