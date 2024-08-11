extern crate proc_macro;

use proc_macro::TokenStream;

#[proc_macro_derive(Parsable)]
pub fn parsable_derive(input: TokenStream) -> TokenStream {
    todo!()
}