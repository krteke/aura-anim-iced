//! Derive macros for Aura animation values.

use proc_macro::TokenStream;
use proc_macro_crate::{FoundCrate, crate_name};
use quote::{format_ident, quote};
use syn::{
    Attribute, Data, DeriveInput, Fields, Generics, Ident, Member, Path, Type, Visibility,
    parse_macro_input, parse_quote,
};

/// Derives field-by-field interpolation for a struct.
///
/// Every field must implement `Animatable`. Named, tuple, and unit structs are
/// supported.
#[proc_macro_derive(Animatable, attributes(animatable))]
pub fn derive_animatable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// Creates a type-safe descriptor for a struct field.
///
/// The input uses field access syntax, for example `field!(Position::x)` or
/// `field!(Offset::0)`.
#[proc_macro]
pub fn field(input: TokenStream) -> TokenStream {
    expand_field(input.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

fn expand(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let base = crate_base_path();
    let via = via_core();

    let field_path = if via {
        quote!(#base::core::field)
    } else {
        quote!(#base::field)
    };

    let traits_path = if via {
        quote!(#base::core::traits)
    } else {
        quote!(#base::traits)
    };

    let interpolate_path = if via {
        quote!(#base::core::interpolate)
    } else {
        quote!(#base::interpolate)
    };

    let name = input.ident;
    let visibility = input.vis;
    let generated_fields_type = fields_name(&input.attrs, &name)?;
    let Data::Struct(data) = input.data else {
        return Err(syn::Error::new_spanned(
            name,
            "Animatable can only be derived for structs",
        ));
    };

    let descriptor_generics = input.generics.clone();
    let mut interpolation_generics = input.generics;
    let field_types = data
        .fields
        .iter()
        .map(|field| field.ty.clone())
        .collect::<Vec<_>>();
    let where_clause = interpolation_generics.make_where_clause();
    for field_type in &field_types {
        where_clause
            .predicates
            .push(parse_quote!(#field_type: #traits_path::Animatable));
    }
    let (impl_generics, type_generics, where_clause) = interpolation_generics.split_for_impl();

    let interpolate_body = match &data.fields {
        Fields::Named(fields) => {
            let names = fields
                .named
                .iter()
                .map(|field| field.ident.as_ref().unwrap())
                .collect::<Vec<_>>();
            quote! {
                Self {
                    #(
                        #names: #traits_path::Interpolate::interpolate_progress(
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
                        #traits_path::Interpolate::interpolate_progress(
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

    let field_descriptors = expand_field_descriptors(
        &field_path,
        &name,
        &visibility,
        &descriptor_generics,
        &data.fields,
        &generated_fields_type,
    );

    let interpolate_impl = quote! {
        impl #impl_generics #traits_path::Interpolate for #name #type_generics #where_clause {
            fn interpolate_progress(
                from: &Self,
                to: &Self,
                progress: #interpolate_path::InterpolationProgress,
            ) -> Self {
                #interpolate_body
            }
        }
    };

    Ok(quote! {
        #interpolate_impl
        #field_descriptors
    })
}

fn expand_field_descriptors(
    path: &proc_macro2::TokenStream,
    struct_name: &Ident,
    visibility: &Visibility,
    generics: &Generics,
    fields: &Fields,
    generated_type: &Ident,
) -> proc_macro2::TokenStream {
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();
    let struct_type = quote!(#struct_name #type_generics);
    let descriptor_constants = match fields {
        Fields::Named(fields) => fields
            .named
            .iter()
            .map(|field| {
                let field_visibility = &field.vis;
                let member = field.ident.as_ref().expect("named field has an identifier");
                let field_type = &field.ty;

                quote! {
                    #[allow(non_upper_case_globals)]
                    #field_visibility const #member: #path::Field<#struct_type, #field_type> =
                        #path::Field::new(
                            stringify!(#member),
                            |value: &#struct_type| &value.#member,
                            |value: &mut #struct_type| &mut value.#member,
                        );
                }
            })
            .collect::<Vec<_>>(),
        Fields::Unnamed(fields) => fields
            .unnamed
            .iter()
            .enumerate()
            .map(|(index, field)| {
                let field_visibility = &field.vis;
                let descriptor_name = format_ident!("_{index}");
                let field_index = syn::Index::from(index);
                let field_type = &field.ty;

                quote! {
                    #[allow(non_upper_case_globals)]
                    #field_visibility const #descriptor_name: #path::Field<#struct_type, #field_type> =
                        #path::Field::new(
                            stringify!(#field_index),
                            |value: &#struct_type| &value.#field_index,
                            |value: &mut #struct_type| &mut value.#field_index,
                        );
                }
            })
            .collect::<Vec<_>>(),
        Fields::Unit => Vec::new(),
    };

    quote! {
        #[doc = concat!("Field descriptors generated for [`", stringify!(#struct_name), "`].")]
        #visibility struct #generated_type #generics {
            __aura_anim_marker: ::core::marker::PhantomData<fn() -> #struct_type>,
        }

        impl #impl_generics #generated_type #type_generics #where_clause
        {
            #(#descriptor_constants)*
        }
    }
}

fn fields_name(attributes: &[Attribute], struct_name: &Ident) -> syn::Result<Ident> {
    let mut name = format_ident!("{struct_name}Fields");
    let mut configured = false;

    for attribute in attributes {
        if !attribute.path().is_ident("animatable") {
            continue;
        }

        attribute.parse_nested_meta(|meta| {
            if !meta.path.is_ident("fields") {
                return Err(meta.error("unsupported animatable option"));
            }
            if configured {
                return Err(meta.error("field descriptor name was already configured"));
            }

            let path: Path = meta.value()?.parse()?;
            let Some(identifier) = path.get_ident() else {
                return Err(meta.error("field descriptor name must be a single identifier"));
            };
            name = identifier.clone();
            configured = true;
            Ok(())
        })?;
    }

    Ok(name)
}

fn expand_field(input: proc_macro2::TokenStream) -> syn::Result<proc_macro2::TokenStream> {
    let tokens = input.into_iter().collect::<Vec<_>>();
    let separator = tokens
        .windows(2)
        .enumerate()
        .filter(|(_, pair)| {
            matches!(&pair[0], proc_macro2::TokenTree::Punct(punct) if punct.as_char() == ':')
                && matches!(&pair[1], proc_macro2::TokenTree::Punct(punct) if punct.as_char() == ':')
        })
        .map(|(index, _)| index)
        .next_back()
        .ok_or_else(|| {
            syn::Error::new(
                proc_macro2::Span::call_site(),
                "expected a field path such as Position::x",
            )
        })?;

    let field_type = syn::parse2::<Type>(tokens[..separator].iter().cloned().collect())?;
    let member = syn::parse2::<Member>(tokens[separator + 2..].iter().cloned().collect())?;

    let base = crate_base_path();
    let via = via_core();

    let field_path = if via {
        quote!(#base::core::field)
    } else {
        quote!(#base::field)
    };

    let name = match &member {
        Member::Named(identifier) => identifier.to_string(),
        Member::Unnamed(index) => index.index.to_string(),
    };

    Ok(quote! {
        #field_path::Field::new(
            #name,
            |value: &#field_type| &value.#member,
            |value: &mut #field_type| &mut value.#member,
        )
    })
}

fn crate_base_path() -> proc_macro2::TokenStream {
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

fn via_core() -> bool {
    for package in ["aura-anim-core", "aura-anim"] {
        if let Ok(found) = crate_name(package) {
            if let (FoundCrate::Itself | FoundCrate::Name(_), "aura-anim-core") = (&found, package)
            {
                return false;
            }

            return true;
        }
    }

    true
}
