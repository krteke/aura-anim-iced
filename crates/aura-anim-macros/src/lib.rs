//! Derive macros for Aura animation values.

use proc_macro::TokenStream;
use proc_macro_crate::{FoundCrate, crate_name};
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Fields, parse_macro_input, parse_quote};

/// Derives field-by-field interpolation for a struct.
///
/// Every field must implement `Animatable`. Named, tuple, and unit structs are
/// supported.
#[proc_macro_derive(Animatable)]
pub fn derive_animatable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

fn expand(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let path = crate_path();
    let name = input.ident;
    let Data::Struct(data) = input.data else {
        return Err(syn::Error::new_spanned(
            name,
            "Animatable can only be derived for structs",
        ));
    };

    let mut generics = input.generics;
    let field_types = data
        .fields
        .iter()
        .map(|field| field.ty.clone())
        .collect::<Vec<_>>();
    let where_clause = generics.make_where_clause();
    for field_type in &field_types {
        where_clause
            .predicates
            .push(parse_quote!(#field_type: #path::Animatable));
    }
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();

    let interpolate_body = match data.fields {
        Fields::Named(fields) => {
            let names = fields
                .named
                .iter()
                .map(|field| field.ident.as_ref().unwrap())
                .collect::<Vec<_>>();
            quote! {
                Self {
                    #(
                        #names: #path::Interpolate::interpolate_progress(
                            &from.#names,
                            &to.#names,
                            progress,
                        )
                    ),*
                }
            }
        }
        Fields::Unnamed(fields) => {
            let indexes = (0..fields.unnamed.len())
                .map(syn::Index::from)
                .collect::<Vec<_>>();
            quote! {
                Self(
                    #(
                        #path::Interpolate::interpolate_progress(
                            &from.#indexes,
                            &to.#indexes,
                            progress,
                        )
                    ),*
                )
            }
        }
        Fields::Unit => quote!(Self),
    };

    let interpolate_impl = quote! {
        impl #impl_generics #path::Interpolate for #name #type_generics #where_clause {
            fn interpolate_progress(
                from: &Self,
                to: &Self,
                progress: #path::InterpolationProgress,
            ) -> Self {
                #interpolate_body
            }
        }
    };

    Ok(interpolate_impl)
}

fn crate_path() -> proc_macro2::TokenStream {
    for package in ["aura-anim-core", "aura-anim"] {
        if let Ok(found) = crate_name(package) {
            return match found {
                FoundCrate::Itself if package == "aura-anim-core" => quote!(crate),
                FoundCrate::Itself => quote!(::aura_anim),
                FoundCrate::Name(name) => {
                    let ident = format_ident!("{name}");
                    quote!(::#ident)
                }
            };
        }
    }

    quote!(::aura_anim)
}
