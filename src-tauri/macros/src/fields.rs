use heck::ToPascalCase;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Ident, ItemStruct};

pub fn fields_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemStruct);
    let struct_name = &input.ident;
    let enum_name = format_ident!("{}Fields", struct_name);
    let fields = &input.fields;

    let field_names = fields.iter().map(|f| &f.ident);
    let field_names_string: Vec<String> = fields
        .iter()
        .map(|f| f.ident.as_ref().unwrap().to_string())
        .collect();

    let pascal_case_field_names: Vec<Ident> = fields
        .iter()
        .map(|f| {
            let ident = f.ident.as_ref().unwrap();
            let pascal_case = ident.to_string().to_pascal_case();
            syn::Ident::new(&pascal_case, ident.span())
        })
        .collect();

    let field_types = fields.iter().map(|f| &f.ty);

    quote! {
        #input

        #[derive(serde::Serialize, serde::Deserialize, Debug, Clone, specta::Type)]
        pub enum #enum_name {
           #(#pascal_case_field_names(#field_types)),*
        }

        impl Into<polodb_core::bson::Bson> for #enum_name {
            fn into(self) -> polodb_core::bson::Bson {
                match self {
                    #(#enum_name::#pascal_case_field_names(value) => polodb_core::bson::bson!({ #field_names_string: value })),*
                }
            }
        }

        impl #struct_name {
            pub fn apply(&mut self, state: #enum_name) {
                match state {
                    #(#enum_name::#pascal_case_field_names(value) => self.#field_names = value),*
                };
            }
        }
    }
    .into()
}
