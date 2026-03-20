// file: plugin-macro/src/lib.rs
use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemStruct, parse_macro_input};

pub(crate) fn _plugin_export(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemStruct);
    let struct_name = &input.ident;

    let expanded = quote! {
        #input

        #[unsafe(no_mangle)]
        pub fn plugin() -> Box<dyn ::plugin::Plugin> {
            Box::new(#struct_name)
        }
    };
    TokenStream::from(expanded)
}
