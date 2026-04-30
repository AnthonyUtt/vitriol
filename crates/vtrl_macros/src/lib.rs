use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, ItemImpl, Type, parse_macro_input};

#[proc_macro_attribute]
pub fn component(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let name = &input.ident;
    let component_name = name.to_string();

    // Extract struct fields for script registration
    let fields = match &input.data {
        syn::Data::Struct(data) => match &data.fields {
            syn::Fields::Named(fields) => fields.named.iter().collect(),
            _ => Vec::new(),
        },
        _ => Vec::new(),
    };

    let field_registrations = fields.iter().map(|field| {
        let field_name = field.ident.as_ref().unwrap();
        let field_name_str = field_name.to_string();
        let field_type = &field.ty;

        quote! {
            engine.register_get_set(
                #field_name_str,
                |obj: &mut #name| -> #field_type {
                    obj.#field_name.clone()
                },
                |obj: &mut #name, val: #field_type| {
                    obj.#field_name = val;
                }
            );
        }
    });

    // Extract enum variants for script registration
    let variants = match &input.data {
        syn::Data::Enum(data) => data
            .variants
            .iter()
            .map(|variant| {
                let variant_name = &variant.ident;
                let variant_str = variant_name.to_string();

                match &variant.fields {
                    syn::Fields::Unit => {
                        quote! {
                            engine.register_get(
                                #variant_str,
                                |_: &mut #name| -> #name { #name::#variant_name },
                            );
                        }
                    }
                    _ => quote! {},
                }
            })
            .collect::<Vec<_>>(),
        _ => Vec::new(),
    };

    let variant_registrations = if !variants.is_empty() {
        quote! {
            let mut module = vtrl_common::prelude::rhai::Module::new();
            #(#variants)*

            engine.register_static_module(#component_name, module.into());
        }
    } else {
        quote! {}
    };

    quote! {
        #[derive(vtrl_common::prelude::serde::Serialize, vtrl_common::prelude::serde::Deserialize, Clone)]
        #input

        impl Component for #name {
            fn name() -> &'static str { #component_name }
            fn as_any(&self) -> &dyn ::std::any::Any { self }
            fn as_any_mut(&mut self) -> &mut dyn ::std::any::Any { self }
            fn register_script_api(engine: &mut vtrl_common::prelude::rhai::Engine) {
                engine.register_type_with_name::<#name>(#component_name);
                #(#field_registrations)*
                #variant_registrations
            }
        }

        vtrl_common::prelude::inventory::submit! {
            ComponentRegistration::new::<#name>(
                #component_name,
                Some(#name::register_script_api),
            )
        }
    }
    .into()
}

#[proc_macro_attribute]
pub fn asset(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemImpl);

    let ty = match &*input.self_ty {
        Type::Path(p) => p,
        _ => {
            return syn::Error::new_spanned(
                &input.self_ty,
                "#[asset] only supports `impl Asset for <TypePath>`",
            )
            .to_compile_error()
            .into();
        }
    };
    let asset_name = ty
        .path
        .segments
        .last()
        .map(|s| s.ident.to_string())
        .unwrap_or_default();

    quote! {
        #input

        vtrl_common::prelude::inventory::submit! {
            vtrl_common::prelude::AssetRegistration::new::<#ty>(#asset_name)
        }
    }
    .into()
}

#[proc_macro_attribute]
pub fn scriptable(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let name = &input.ident;
    let name_str = name.to_string();

    // Extract struct fields for script registration
    let fields = match &input.data {
        syn::Data::Struct(data) => match &data.fields {
            syn::Fields::Named(fields) => fields.named.iter().collect(),
            _ => Vec::new(),
        },
        _ => Vec::new(),
    };

    let field_registrations = fields.iter().map(|field| {
        let field_name = field.ident.as_ref().unwrap();
        let field_name_str = field_name.to_string();
        let field_type = &field.ty;

        quote! {
            engine.register_get_set(
                #field_name_str,
                |obj: &mut #name| -> #field_type {
                    obj.#field_name.clone()
                },
                |obj: &mut #name, val: #field_type| {
                    obj.#field_name = val;
                }
            );
        }
    });

    // Extract enum variants for script registration
    let variants = match &input.data {
        syn::Data::Enum(data) => data
            .variants
            .iter()
            .map(|variant| {
                let variant_name = &variant.ident;
                let variant_str = variant_name.to_string();

                match &variant.fields {
                    syn::Fields::Unit => {
                        quote! {
                            module.set_var(#variant_str, #name::#variant_name);
                        }
                    }
                    _ => quote! {},
                }
            })
            .collect::<Vec<_>>(),
        _ => Vec::new(),
    };

    let variant_registrations = if !variants.is_empty() {
        quote! {
            let mut module = vtrl_common::prelude::rhai::Module::new();
            #(#variants)*

            engine.register_static_module(#name_str, module.into());
        }
    } else {
        quote! {}
    };

    quote! {
        #input

        impl Scriptable for #name {
            fn register_script_api(engine: &mut vtrl_common::prelude::rhai::Engine) {
                engine.register_type_with_name::<#name>(#name_str);
                #(#field_registrations)*
                #variant_registrations
            }
        }

        vtrl_common::prelude::inventory::submit! {
            ScriptableRegistration::new(#name::register_script_api)
        }
    }
    .into()
}
