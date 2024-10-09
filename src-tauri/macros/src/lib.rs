extern crate proc_macro;

mod anyhow_to_string;
mod fields;
mod model_to_event;

use anyhow_to_string::anyhow_to_string_impl;
use fields::fields_impl;
use model_to_event::model_to_event_impl;
use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn anyhow_to_string(_attr: TokenStream, item: TokenStream) -> TokenStream {
    anyhow_to_string_impl(item)
}

#[proc_macro_attribute]
pub fn model_to_event(_attr: TokenStream, item: TokenStream) -> TokenStream {
    model_to_event_impl(item)
}

#[proc_macro_attribute]
pub fn fields(_attr: TokenStream, item: TokenStream) -> TokenStream {
    fields_impl(item)
}
