use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, FnArg, ItemFn, Pat, ReturnType};

#[proc_macro_attribute]
pub fn command(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);
    let fn_name = &input_fn.sig.ident;
    let fn_name_str = fn_name.to_string();

    // Extract function arguments (skip self params)
    let args: Vec<_> = input_fn
        .sig
        .inputs
        .iter()
        .filter_map(|arg| {
            if let FnArg::Typed(pat_type) = arg {
                if let Pat::Ident(ident) = pat_type.pat.as_ref() {
                    let name = &ident.ident;
                    let ty = &pat_type.ty;
                    return Some((name.clone(), ty.clone()));
                }
            }
            None
        })
        .collect();

    // Generate argument extraction code
    let arg_extractions: Vec<_> = args
        .iter()
        .map(|(name, ty)| {
            let name_str = name.to_string();
            quote! {
                let #name: #ty = serde_json::from_value(
                    args.get(#name_str)
                        .cloned()
                        .unwrap_or(serde_json::Value::Null)
                ).map_err(|e| runtime_ipc::IpcError::InvalidArgs(
                    format!("field '{}': {}", #name_str, e)
                ))?;
            }
        })
        .collect();

    let arg_names: Vec<_> = args.iter().map(|(n, _)| n.clone()).collect();

    // Check if return type is Result
    let has_result = matches!(&input_fn.sig.output, ReturnType::Type(_, ty) if {
        let s = quote!(#ty).to_string();
        s.contains("Result")
    });

    let call_and_return = if has_result {
        quote! {
            let result = #fn_name(#(#arg_names),*)
                .map_err(|e| runtime_ipc::IpcError::HandlerError(e.to_string()))?;
            serde_json::to_value(result).map_err(runtime_ipc::IpcError::Serialization)
        }
    } else {
        quote! {
            let result = #fn_name(#(#arg_names),*);
            serde_json::to_value(result).map_err(runtime_ipc::IpcError::Serialization)
        }
    };

    let handler_name = syn::Ident::new(
        &format!("__command_handler_{}", fn_name),
        fn_name.span(),
    );

    let expanded = quote! {
        #input_fn

        pub struct #handler_name;

        impl runtime_ipc::command::CommandHandler for #handler_name {
            fn handle(&self, args: serde_json::Value) -> Result<serde_json::Value, runtime_ipc::IpcError> {
                #(#arg_extractions)*
                #call_and_return
            }
        }

        impl #handler_name {
            pub const NAME: &'static str = #fn_name_str;
        }
    };

    TokenStream::from(expanded)
}
