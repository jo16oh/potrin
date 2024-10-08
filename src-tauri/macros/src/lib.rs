extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, GenericArgument, ItemFn, PathArguments, ReturnType, Type};

#[proc_macro_attribute]
pub fn anyhow_to_string(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    // 関数のシグネチャを取得
    let sig = &input.sig;
    let name = &sig.ident;
    let block = &input.block;
    let vis = &input.vis;

    // 戻り値の型がanyhow::Resultであることをチェック
    if let ReturnType::Type(_, ty) = &sig.output {
        if let Type::Path(type_path) = &**ty {
            let segments = &type_path.path.segments;
            if segments.len() == 2 && segments[0].ident == "anyhow" && segments[1].ident == "Result"
            {
                // 元の関数の戻り値の型を取得
                let inner_ty =
                    if let PathArguments::AngleBracketed(ref args) = segments[1].arguments {
                        if let Some(GenericArgument::Type(ref inner_ty)) = args.args.first() {
                            inner_ty
                        } else {
                            return syn::Error::new_spanned(
                                &sig.output,
                                "Expected a type argument for Result",
                            )
                            .to_compile_error()
                            .into();
                        }
                    } else {
                        return syn::Error::new_spanned(
                            &sig.output,
                            "Expected angle bracketed arguments for Result",
                        )
                        .to_compile_error()
                        .into();
                    };

                // 新しい戻り値の型をResult<T, String>に変更
                let new_output = quote! { -> Result<#inner_ty, String> };

                // 引数リストを取得
                let inputs = &sig.inputs;

                // 条件を満たしている場合は元の関数をラップして新しい関数を生成
                let gen = quote! {
                    #vis fn #name(#inputs) #new_output {
                        let result: anyhow::Result<#inner_ty> = (|| #block)();
                        match result {
                            Ok(val) => Ok(val),
                            Err(e) => Err(e.to_string()),
                        }
                    }
                };
                return gen.into();
            }
        }
    }

    // 条件を満たしていない場合はエラーを返す
    syn::Error::new_spanned(&sig.output, "Return type must be anyhow::Result<T>")
        .to_compile_error()
        .into()
}
