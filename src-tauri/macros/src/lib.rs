extern crate proc_macro;

mod anyhow_to_string;

use anyhow_to_string::anyhow_to_string_impl;
use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn anyhow_to_string(_attr: TokenStream, item: TokenStream) -> TokenStream {
    anyhow_to_string_impl(item)
}
