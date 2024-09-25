use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemStruct};

pub fn into_bson_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemStruct);
    let name = &input.ident;
    let fields = &input.fields;

    let field_names = fields.iter().map(|f| &f.ident);
    let field_names_string = fields.iter().map(|f| {
        let ident = f.ident.as_ref().unwrap();
        ident.to_string()
    });

    quote! {
        impl Into<polodb_core::bson::Bson> for #name {
            fn into(self) -> polodb_core::bson::Bson {
                polodb_core::bson::bson!({
                    #(#field_names_string: self.#field_names),*
                })
            }
        }
    }
    .into()
}
