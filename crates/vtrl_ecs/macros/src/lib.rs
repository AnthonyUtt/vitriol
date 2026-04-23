use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, LitStr, parse_macro_input};

#[proc_macro_derive(Component, attributes(component))]
pub fn derive_component(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ident = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let mut component_name: Option<LitStr> = None;
    for attr in &input.attrs {
        if !attr.path().is_ident("component") {
            continue;
        }
        let parse_result = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("name") {
                component_name = Some(meta.value()?.parse()?);
                Ok(())
            } else {
                Err(meta.error("unsupported component attribute; expected `name = \"...\"`"))
            }
        });
        if let Err(err) = parse_result {
            return err.to_compile_error().into();
        }
    }

    let name_str = component_name
        .map(|s| s.value())
        .unwrap_or_else(|| ident.to_string());

    quote! {
        impl #impl_generics ::vtrl_ecs::prelude::Component for #ident #ty_generics #where_clause {
            fn name() -> &'static str { #name_str }
            fn as_any(&self) -> &dyn ::std::any::Any { self }
            fn as_any_mut(&mut self) -> &mut dyn ::std::any::Any { self }
        }
    }
    .into()
}
