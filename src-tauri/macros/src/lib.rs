extern crate proc_macro;

mod anyhow_to_string;
mod fields;
mod into_bson;
mod table_change_event;

use anyhow_to_string::anyhow_to_string_impl;
use fields::fields_impl;
use into_bson::into_bson_impl;
use proc_macro::TokenStream;
use table_change_event::table_change_event_impl;

#[proc_macro_attribute]
pub fn anyhow_to_string(_attr: TokenStream, item: TokenStream) -> TokenStream {
    anyhow_to_string_impl(item)
}

#[proc_macro_attribute]
pub fn table_change_event(_attr: TokenStream, item: TokenStream) -> TokenStream {
    table_change_event_impl(item)
}

#[proc_macro_attribute]
pub fn fields(_attr: TokenStream, item: TokenStream) -> TokenStream {
    fields_impl(item)
}

#[proc_macro_derive(Bson)]
pub fn into_bson(item: TokenStream) -> TokenStream {
    into_bson_impl(item)
}
