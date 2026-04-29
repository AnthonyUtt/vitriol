use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, ItemImpl, Type, parse_macro_input};

#[proc_macro_attribute]
pub fn component(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let name = &input.ident;
    let component_name = name.to_string();

    quote! {
        #[derive(vtrl_common::prelude::serde::Serialize, vtrl_common::prelude::serde::Deserialize)]
        #input

        impl Component for #name {
            fn name() -> &'static str { #component_name }
            fn as_any(&self) -> &dyn ::std::any::Any { self }
            fn as_any_mut(&mut self) -> &mut dyn ::std::any::Any { self }
        }

        vtrl_common::prelude::inventory::submit! {
            ComponentRegistration::new::<#name>(#component_name)
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
