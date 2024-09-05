use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, ItemStruct};

pub fn table_change_event_impl(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);
    let table_name = &input.ident;
    let event_name = format_ident!("{}ChangeEvent", table_name);

    quote! {
        #input

        #[derive(Serialize, Deserialize, Debug, Clone, Type, Event)]
        pub struct #event_name {
            operation: Operation,
            rows_changed: Vec<#table_name>,
        }

        impl #event_name {
            pub fn new(operation: Operation, rows: &[#table_name]) -> Self {
                #event_name {
                    operation,
                    rows_changed: rows.to_vec(),
                }
            }
        }
    }
    .into()
}
