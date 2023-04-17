use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::spanned::Spanned;
use syn::{
    parse_macro_input, Fields, Ident, Item, ItemEnum, ItemStruct, Token, Type, Visibility,
};

/// Make a `struct` or `enum` ready for use as a message.
/// ```
/// use protocol_macros::message_part;
/// #[message_part]
/// struct Mpv {
///     asd: i32,
/// }
/// ```
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

/// Add `pub` to all fields in the struct
fn pubify_struct(struc: &mut ItemStruct) {
    match struc.fields {
        Fields::Named(ref mut fields) => fields.named.iter_mut().for_each(|field| {
            field.vis = Visibility::Public(Token![pub](Span::call_site()))
        }),
        _ => (),
    }
}

/// Generate different kinds of functions and trait impls to make this `enum` easier to
/// use as a message aggregator.
/// ```
/// use protocol_macros::message_part;
/// #[message_part]
/// struct Mpv {
///     asd: i32,
/// }
///
/// use protocol_macros::message_aggregator;
/// #[message_aggregator]
/// enum Message {
///     Mpv(Mpv),
/// }
/// ```
#[proc_macro_attribute]
pub fn message_aggregator(direction: TokenStream, enu: TokenStream) -> TokenStream {
    let enu = parse_macro_input!(enu as ItemEnum);

    let generated = create_code(&enu);

    let tokens = quote! {
        #[protocol_macros::message_part]
        #enu
        #generated
    };

    tokens.into()
}

// TODO: add toserver/toclient as argument
fn create_code(enu: &ItemEnum) -> proc_macro2::TokenStream {
    let enum_name = &enu.ident;
    enu.variants
        .iter()
        .map(|variant| {
            assert!(variant.discriminant.is_none());
            let variant_name = &variant.ident;

            match &variant.fields {
                Fields::Unnamed(variant) => {
                    if variant.unnamed.len() != 1 {
                        return syn::Error::new(
                            variant_name.span(),
                            "Must have exactly one field",
                        )
                        .to_compile_error();
                    }

                    let variant = variant
                        .unnamed
                        .first()
                        .expect("number of variants already checked");

                    let variant_field = &variant.ty;

                    let reexport = create_reexport(enum_name, variant_name);
                    let into_parent =
                        create_into_parent(enum_name, variant_name, variant_field);

                    quote! {
                        #reexport
                        #into_parent
                    }
                }
                Fields::Named(_) => {
                    return syn::Error::new(
                        variant_name.span(),
                        "Named fields are not supported",
                    )
                    .to_compile_error()
                }
                Fields::Unit => create_reexport(enum_name, variant_name),
            }
        })
        .collect()
}

fn create_into_parent(
    enum_name: &Ident,
    variant_name: &Ident,
    variant_field: &Type,
) -> proc_macro2::TokenStream {
    quote! {
        impl From<#variant_field> for #enum_name {
            fn from(sub: #variant_field) -> Self {
                <#enum_name>::#variant_name(sub)
            }
        }
    }
}

fn create_into_trait(
    enum_name: &Ident,
    variant_name: &Ident,
    variant_field: &Type,
    trait_name: &Type,
) -> proc_macro2::TokenStream {
    quote! {
        impl From<#variant_field> for #trait_name {
            fn from(sub: #variant_field) -> Self {
                let parent: #enum_name = sub.into();
                parent.into()
            }
        }
    }
}

fn create_reexport(enum_name: &Ident, variant_name: &Ident) -> proc_macro2::TokenStream {
    quote! {
        pub use #enum_name::#variant_name;
    }
}

fn create_getters(enum_name: &Ident, variant_name: &Ident) -> proc_macro2::TokenStream {
    quote! {}
}

fn create_isers(enum_name: &Ident, variant_name: &Ident) -> proc_macro2::TokenStream {
    quote! {}
}
