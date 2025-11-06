pub mod codec;
pub mod r#enum;
pub mod field;
pub mod r#struct;

pub use codec::*;
pub use r#enum::*;
pub use field::*;
pub use r#struct::*;
use syn::{DeriveInput, spanned::Spanned as _};

pub trait BinarySchema {
    fn decode(&self, _ctx: &DecodeContext) -> syn::Result<proc_macro2::TokenStream> {
        Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "Decode not implemented",
        ))
    }
    fn encode(&self, _ctx: &EncodeContext) -> syn::Result<proc_macro2::TokenStream> {
        Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "Encode not implemented",
        ))
    }
    fn measure_fixed(&self) -> syn::Result<proc_macro2::TokenStream> {
        Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "Measure fixed not implemented",
        ))
    }
    fn measure(&self, _ctx: &MeasureContext) -> syn::Result<proc_macro2::TokenStream> {
        Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "Measure not implemented",
        ))
    }
}

#[derive(Clone)]
pub struct DecodeContext {
    pub encoded: proc_macro2::TokenStream,
    pub offset: proc_macro2::TokenStream,
}

#[derive(Clone)]
pub struct EncodeContext {
    pub wrapper: proc_macro2::TokenStream,
    pub decoded: proc_macro2::TokenStream,
    pub encoded: proc_macro2::TokenStream,
    pub offset: proc_macro2::TokenStream,
}

#[derive(Clone)]
pub struct MeasureContext {
    pub wrapper: proc_macro2::TokenStream,
    pub decoded: proc_macro2::TokenStream,
}

pub fn interpret_derive_schema(input: &DeriveInput) -> syn::Result<Box<dyn BinarySchema>> {
    match &input.data {
        syn::Data::Struct(_) => interpret_struct_schema(input),
        syn::Data::Enum(_) => interpret_enum_schema(input),
        _ => Err(syn::Error::new(
            input.span(),
            "Only structs and enums are supported",
        )),
    }
}
