use once_cell::sync::Lazy;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use regex::Regex;
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
pub fn message_part(attr: TokenStream, item: TokenStream) -> TokenStream {
    if !attr.is_empty() {
        let attr: proc_macro2::TokenStream = attr.into();
        return syn::Error::new(attr.span(), "no arguments allowed")
            .into_compile_error()
            .into();
    }

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
/// mod message {
///     use protocol_macros::message_aggregator;
///     #[message_aggregator]
///     enum Message {
///         Mpv(mpv::Mpv),
///         Control(mpv::Control),
///         FilesThing,
///     }
///
///     pub mod mpv {
///         use protocol_macros::{message_part, message_aggregator};
///         #[message_part]
///         struct Mpv {
///             asd: i32,
///         }
///
///         #[message_aggregator(super::Message)]
///         enum Control {
///             Play(play::Play),
///         }
///
///         pub mod play {
///             use protocol_macros::message_part;
///             #[message_part]
///             struct Play;
///         }
///     }
/// }
///
/// use message::*;
/// assert!(Message::FilesThing.is_files_thing());
/// let mpv = mpv::Mpv{asd: 123};
/// let msg: Message = mpv.clone().into();
/// assert!(msg.is_mpv());
/// assert_eq!(Some(&mpv), msg.borrow_mpv());
/// assert_eq!(Ok(mpv), msg.take_mpv());
///
/// let msg: Message = mpv::play::Play.into();
/// ```
#[proc_macro_attribute]
pub fn message_aggregator(grandparent: TokenStream, enu: TokenStream) -> TokenStream {
    let enu = parse_macro_input!(enu as ItemEnum);
    let grandparent = if !grandparent.is_empty() {
        Some(parse_macro_input!(grandparent as Type))
    } else {
        None
    };

    let generated = create_code(&enu, grandparent.as_ref());

    let tokens = quote! {
        #[protocol_macros::message_part]
        #enu
        #generated
    };

    tokens.into()
}

/// Create additional functions and trait impls for the enum.
fn create_code(enu: &ItemEnum, grandparent: Option<&Type>) -> proc_macro2::TokenStream {
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
                    let functions =
                        create_functions_single(enum_name, variant_name, variant_field);

                    let grand = if let Some(grandparent) = grandparent {
                        create_into_grandparent(enum_name, variant_field, grandparent)
                    } else {
                        quote! {}
                    };

                    quote! {
                        #reexport
                        #into_parent
                        #functions
                        #grand
                    }
                }
                Fields::Named(_) => {
                    return syn::Error::new(
                        variant_name.span(),
                        "Named fields are not supported",
                    )
                    .to_compile_error()
                }
                Fields::Unit => {
                    let reexport = create_reexport(enum_name, variant_name);
                    let functions = create_functions_unit(enum_name, variant_name);

                    quote! {
                        #reexport
                        #functions
                    }
                }
            }
        })
        .collect()
}

/// Create `From` for the direct parent aggregator
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

/// Create `From` for some given parent aggregator
fn create_into_grandparent(
    enum_name: &Ident,
    variant_field: &Type,
    grandparent_name: &Type,
) -> proc_macro2::TokenStream {
    quote! {
        impl From<#variant_field> for #grandparent_name {
            fn from(sub: #variant_field) -> Self {
                let parent: #enum_name = sub.into();
                parent.into()
            }
        }
    }
}

/// Create re-export for enum variants
fn create_reexport(enum_name: &Ident, variant_name: &Ident) -> proc_macro2::TokenStream {
    quote! {
        pub use #enum_name::#variant_name;
    }
}

/// Create accessor functions for enum unit variants
fn create_functions_unit(
    enum_name: &Ident,
    variant_name: &Ident,
) -> proc_macro2::TokenStream {
    let is_name = Ident::new(
        &format!("is_{}", camel2snake(&variant_name.to_string())),
        Span::call_site(),
    );
    quote! {
        impl #enum_name {
            pub fn #is_name(&self) -> bool {
                std::matches!(self, #enum_name::#variant_name)
            }
        }
    }
}

/// Create accessor functions for enum tuple variants with one element
fn create_functions_single(
    enum_name: &Ident,
    variant_name: &Ident,
    variant_field: &Type,
) -> proc_macro2::TokenStream {
    let is_name = Ident::new(
        &format!("is_{}", camel2snake(&variant_name.to_string())),
        Span::call_site(),
    );
    let take_name = Ident::new(
        &format!("take_{}", camel2snake(&variant_name.to_string())),
        Span::call_site(),
    );
    let borrow_name = Ident::new(
        &format!("borrow_{}", camel2snake(&variant_name.to_string())),
        Span::call_site(),
    );

    quote! {
        impl #enum_name {
            pub fn #is_name(&self) -> bool {
                std::matches!(self, #enum_name::#variant_name(_))
            }

            pub fn #take_name(self) -> std::result::Result<#variant_field, #enum_name> {
                match self {
                    #enum_name::#variant_name(inner) => std::result::Result::Ok(inner),
                    _ => std::result::Result::Err(self),
                }
            }

            pub fn #borrow_name(&self) -> std::option::Option<&#variant_field> {
                match self {
                    #enum_name::#variant_name(inner) => std::option::Option::Some(inner),
                    _ => std::option::Option::None,
                }
            }
        }
    }
}

/// Converta a CamelCase name into snake_case
fn camel2snake(camel: &str) -> String {
    static RE: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"([[:lower:]])([[:upper:]])").unwrap());
    RE.replace_all(camel, r"${1}_${2}").to_lowercase()
}

#[test]
fn case_tests() {
    assert_eq!("to_server", camel2snake("ToServer"));
    assert_eq!("to_server", camel2snake("toSERver"));
    assert_eq!("toserver", camel2snake("toserver"));
}
