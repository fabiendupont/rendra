use syn::visit::Visit;
use syn::{FnArg, ItemFn, Pat, ReturnType, Type};

struct CommandVisitor {
    commands: Vec<CommandInfo>,
}

struct CommandInfo {
    name: String,
    args: Vec<(String, String)>,
    return_type: String,
}

impl<'ast> Visit<'ast> for CommandVisitor {
    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        let has_command_attr = node
            .attrs
            .iter()
            .any(|attr| attr.path().is_ident("command"));

        if has_command_attr {
            let name = node.sig.ident.to_string();

            let args: Vec<(String, String)> = node
                .sig
                .inputs
                .iter()
                .filter_map(|arg| {
                    if let FnArg::Typed(pat_type) = arg {
                        let arg_name = if let Pat::Ident(pat_ident) = pat_type.pat.as_ref() {
                            pat_ident.ident.to_string()
                        } else {
                            return None;
                        };
                        let ts_type = rust_type_to_ts(&pat_type.ty);
                        Some((arg_name, ts_type))
                    } else {
                        None
                    }
                })
                .collect();

            let return_type = match &node.sig.output {
                ReturnType::Default => "void".to_string(),
                ReturnType::Type(_, ty) => extract_return_type(ty),
            };

            self.commands.push(CommandInfo {
                name,
                args,
                return_type,
            });
        }

        syn::visit::visit_item_fn(self, node);
    }
}

fn rust_type_to_ts(ty: &Type) -> String {
    let type_str = quote::quote!(#ty).to_string();
    let type_str = type_str.replace(' ', "");

    match type_str.as_str() {
        "String" | "&str" => "string".to_string(),
        "bool" => "boolean".to_string(),
        "i8" | "i16" | "i32" | "i64" | "u8" | "u16" | "u32" | "u64" | "f32" | "f64" => {
            "number".to_string()
        }
        _ => "any".to_string(),
    }
}

fn extract_return_type(ty: &Type) -> String {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            if segment.ident == "Result" {
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(syn::GenericArgument::Type(ok_type)) = args.args.first() {
                        return rust_type_to_ts(ok_type);
                    }
                }
            }
        }
    }
    rust_type_to_ts(ty)
}

pub fn generate_bindings(source: &str) -> String {
    let syntax = syn::parse_file(source).expect("failed to parse Rust source");

    let mut visitor = CommandVisitor {
        commands: Vec::new(),
    };
    visitor.visit_file(&syntax);

    let mut output = String::new();

    output.push_str(
        "declare const __runtime: {\n  invoke(cmd: string, args?: Record<string, unknown>): \
         Promise<unknown>;\n  on(event: string, handler: (...args: unknown[]) => void): void;\n\
         };\n\n",
    );

    output.push_str("export const commands = {\n");

    for (i, cmd) in visitor.commands.iter().enumerate() {
        let params: Vec<String> = cmd
            .args
            .iter()
            .map(|(name, ts_type)| format!("{}: {}", name, ts_type))
            .collect();
        let params_str = params.join(", ");

        let arg_obj = if cmd.args.is_empty() {
            String::new()
        } else {
            let fields: Vec<String> = cmd.args.iter().map(|(name, _)| name.clone()).collect();
            format!(", {{ {} }}", fields.join(", "))
        };

        output.push_str(&format!(
            "  async {}({}): Promise<{}> {{\n    return __runtime.invoke('{}'{}) as \
             Promise<{}>;\n  }}",
            cmd.name, params_str, cmd.return_type, cmd.name, arg_obj, cmd.return_type
        ));

        if i < visitor.commands.len() - 1 {
            output.push_str(",\n");
        } else {
            output.push('\n');
        }
    }

    output.push_str("};\n");

    output
}
