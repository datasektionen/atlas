use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Expr, Fields};

#[proc_macro_derive(AceInstance, attributes(db, constants))]
pub fn derive_ace_instance(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = input.ident;

    let mut const_keys = Vec::new();
    let mut const_vals = Vec::new();

    for attr in &input.attrs {
        if attr.path().is_ident("constants") {
            let _ = attr.parse_nested_meta(|meta| {
                let name = meta.path.get_ident().unwrap().to_string();
                let value: Expr = meta.value()?.parse()?;

                const_keys.push(name);
                const_vals.push(value);
                Ok(())
            });
        }
    }

    let (field_names, db_paths): (Vec<_>, Vec<_>) = match input.data {
        Data::Struct(data_struct) => match data_struct.fields {
            Fields::Named(fields_named) => fields_named
                .named
                .iter()
                .map(|f| {
                    let ident = f.ident.as_ref().unwrap().to_string();
                    let mut db_path = None;

                    for attr in &f.attrs {
                        if attr.path().is_ident("db") {
                            if let Ok(name_val) = attr.meta.require_name_value() {
                                db_path = Some(name_val.value.clone());
                            } else {
                                panic!("Wrong syntax in database path for field {ident}");
                            }
                        }
                    }

                    let path = db_path.unwrap_or_else(|| {
                        panic!("No valid database path for field {ident}");
                    });

                    (ident, path)
                })
                .unzip(),
            _ => panic!("AceInstance only supports structs with named fields"),
        },
        _ => panic!("AceInstance can only be derived on structs"),
    };

    let expanded = quote! {
        impl ace::AceInstance for #struct_name {
            // XXX: cannot be constant due to AceValue::from
            fn constants() -> std::collections::HashMap<&'static str, ace::AceValue> {
                std::collections::HashMap::from([
                    #((#const_keys, crate::AceValue::from(#const_vals))),*
                ])
            }

            const FIELDS: &'static [(&'static str, &'static str)] =
                &[#((#field_names, #db_paths)),*];
        }
    };

    TokenStream::from(expanded)
}
