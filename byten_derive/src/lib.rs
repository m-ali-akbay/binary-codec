use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

mod schema;
use schema::*;

fn _byten(input: TokenStream) -> syn::Result<TokenStream> {
    let expr = build_codec_expr(input)?;
    Ok(quote! { #expr })
}

#[proc_macro]
pub fn byten(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as TokenStream);
    _byten(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

fn _derive_default_codec(input: DeriveInput) -> syn::Result<TokenStream> {
    let ident = &input.ident;
    let generics = &input.generics;

    Ok(quote! {
        impl #generics ::byten::DefaultCodec for #ident #generics {
            type Codec = ::byten::SelfCoded<Self>;
            fn default_codec() -> ::byten::SelfCoded<Self> {
                ::byten::SelfCoded::<Self>::new()
            }
        }
    }
    .into())
}

#[proc_macro_derive(DefaultCodec, attributes(byten))]
pub fn derive_default_codec(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    _derive_default_codec(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

fn _derive_decode_owned(input: DeriveInput) -> syn::Result<TokenStream> {
    let ident = &input.ident;
    let generics = &input.generics;

    let schema = interpret_derive_schema(&input)?;

    let decoded = schema.decode(&DecodeContext {
        encoded: quote! { encoded },
        offset: quote! { offset },
    })?;

    Ok(quote! {
        impl #generics ::byten::Decode<'_> for #ident #generics {
            fn decode(encoded: &'_ [u8], offset: &mut usize) -> Result<Self, ::byten::DecodeError> {
                Ok(#decoded)
            }
        }
    }
    .into())
}

#[proc_macro_derive(DecodeOwned, attributes(byten))]
pub fn derive_decode_owned(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: DeriveInput = syn::parse(input).unwrap();
    _derive_decode_owned(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

fn _derive_decode(input: DeriveInput) -> syn::Result<TokenStream> {
    let ident = &input.ident;
    let generics = &input.generics;

    let schema = interpret_derive_schema(&input)?;

    let decoded = schema.decode(&DecodeContext {
        encoded: quote! { encoded },
        offset: quote! { offset },
    })?;

    Ok(quote! {
        impl #generics ::byten::Decode<'encoded> for #ident #generics {
            fn decode(encoded: &'encoded [u8], offset: &mut usize) -> Result<Self, ::byten::DecodeError> {
                Ok(#decoded)
            }
        }
    }.into())
}

#[proc_macro_derive(Decode, attributes(byten))]
pub fn derive_decode(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: DeriveInput = syn::parse(input).unwrap();
    _derive_decode(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

fn _derive_encode(input: DeriveInput) -> syn::Result<TokenStream> {
    let ident = &input.ident;
    let generics = &input.generics;

    let schema = interpret_derive_schema(&input)?;

    let encoded = schema.encode(&EncodeContext {
        wrapper: quote! { Self },
        decoded: quote! { self },
        encoded: quote! { encoded },
        offset: quote! { offset },
    })?;

    Ok(quote! {
        impl #generics ::byten::Encode for #ident #generics {
            fn encode(&self, encoded: &mut [u8], offset: &mut usize) -> Result<(), ::byten::EncodeError> {
                #encoded
                Ok(())
            }
        }
    }.into())
}

#[proc_macro_derive(Encode, attributes(byten))]
pub fn derive_encode(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: DeriveInput = syn::parse(input).unwrap();
    _derive_encode(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

fn _derive_measure(input: DeriveInput) -> syn::Result<TokenStream> {
    let ident = &input.ident;
    let generics = &input.generics;

    let schema = interpret_derive_schema(&input)?;

    let measured = schema.measure(&MeasureContext {
        wrapper: quote! { Self },
        decoded: quote! { self },
    })?;

    Ok(quote! {
        impl #generics ::byten::Measure for #ident #generics {
            fn measure(&self) -> Result<usize, ::byten::EncodeError> {
                Ok(#measured)
            }
        }
    }
    .into())
}

#[proc_macro_derive(Measure, attributes(byten))]
pub fn derive_measure(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: DeriveInput = syn::parse(input).unwrap();
    _derive_measure(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_derive(MeasureFixed, attributes(byten))]
pub fn derive_measure_fixed(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: DeriveInput = syn::parse(input).unwrap();
    _derive_measure_fixed(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

fn _derive_measure_fixed(input: DeriveInput) -> syn::Result<TokenStream> {
    let ident = &input.ident;
    let generics = &input.generics;

    let schema = interpret_derive_schema(&input)?;

    let measured_fixed = schema.measure_fixed()?;

    Ok(quote! {
        impl #generics ::byten::MeasureFixed for #ident #generics {
            fn measure_fixed() -> usize {
                #measured_fixed
            }
        }

        impl #generics ::byten::Measure for #ident #generics {
            fn measure(&self) -> Result<usize, ::byten::EncodeError> {
                Ok(Self::measure_fixed())
            }
        }
    }
    .into())
}
