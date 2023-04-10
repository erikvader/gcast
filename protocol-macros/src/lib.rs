use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_attribute]
pub fn message_part(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item2: proc_macro2::TokenStream = item.into();
    let tokens = quote! {
        #[derive(Debug, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
        pub #item2
    };
    tokens.into()
}
