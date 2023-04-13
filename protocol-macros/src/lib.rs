use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::spanned::Spanned;
use syn::{parse_macro_input, Fields, Item, ItemStruct, Token, Visibility};

#[proc_macro_attribute]
pub fn message_part(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut item = parse_macro_input!(item as Item);

    match item {
        Item::Enum(_) => (),
        Item::Struct(ref mut item) => pubify_struct(item),
        _ => {
            return syn::Error::new(
                item.span(),
                "message_part can only be used on an enum or a struct",
            )
            .into_compile_error()
            .into()
        }
    }

    let tokens = quote! {
        #[derive(Debug, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
        pub #item
    };

    tokens.into()
}

fn pubify_struct(struc: &mut ItemStruct) {
    match struc.fields {
        Fields::Named(ref mut fields) => fields.named.iter_mut().for_each(|field| {
            field.vis = Visibility::Public(Token![pub](Span::call_site()))
        }),
        _ => (),
    }
}
