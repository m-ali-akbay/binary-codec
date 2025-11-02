use proc_macro2::Span;
use syn::{Fields, FieldsNamed, Ident};
use quote::{ToTokens, quote};

use crate::{build_codec_schema};

use super::{BinarySchema, DecodeContext, EncodeContext, MeasureContext};

pub trait FieldsSchema: BinarySchema {
    fn wildcard_pattern(&self) -> proc_macro2::TokenStream;
}

pub fn interpret_fields_schema(fields: &Fields) -> syn::Result<Box<dyn FieldsSchema>> {
    Ok(match fields {
        Fields::Named(fields) => Box::new(NamedFieldsSchema::interpret(fields)?),
        Fields::Unnamed(fields) => Box::new(UnnamedFieldsSchema::interpret(fields)?),
        Fields::Unit => Box::new(UnitFieldsSchema {}),
    })
}

struct NamedFieldsSchema {
    fields: Vec<(Ident, Box<dyn BinarySchema>)>,
}

impl FieldsSchema for NamedFieldsSchema {
    fn wildcard_pattern(&self) -> proc_macro2::TokenStream {
        quote! { { .. } }
    }
}

impl NamedFieldsSchema {
    fn interpret(fields: &FieldsNamed) -> syn::Result<NamedFieldsSchema> {
        let fields = fields.named.iter().map(|field| {
            let ident = field.ident.clone().ok_or_else(|| {
                syn::Error::new_spanned(
                    field,
                    "Named field must have an identifier",
                )
            })?;
            let codec = build_codec_schema(&field.attrs, Some(&field.ty))?;
            Ok((ident, codec))
        }).collect::<syn::Result<Vec<_>>>()?;
        Ok(NamedFieldsSchema {
            fields,
        })
    }
}

impl BinarySchema for NamedFieldsSchema {
    fn decode(&self, ctx: &DecodeContext) -> syn::Result<proc_macro2::TokenStream> {
        let fields = self.fields.iter().map(|(ident, schema)| {
            let decode = schema.decode(&ctx.clone())?;
            Ok(quote! { #ident: #decode })
        }).collect::<syn::Result<Vec<_>>>()?;
        Ok(quote! { { #(#fields),* } })
    }

    fn encode(&self, ctx: &EncodeContext) -> syn::Result<proc_macro2::TokenStream> {
        let wrapper = &ctx.decoded;
        let type_path = &ctx.wrapper;
        let idents = self.fields.iter().map(|(ident, _)| ident).collect::<Vec<_>>();
        let variables = idents.iter()
            .map(|ident| Ident::new(format!("variant_{}", ident).as_str(), ident.span()))
            .collect::<Vec<_>>();
        let encodes = self.fields.iter().zip(variables.iter()).map(|((_, schema), variable)| {
            schema.encode(&EncodeContext {
                wrapper: quote! {},
                decoded: variable.into_token_stream(),
                encoded: ctx.encoded.clone(),
                offset: ctx.offset.clone(),
            })
        }).collect::<syn::Result<Vec<_>>>()?;
        Ok(quote! { 
            let #type_path { #(#idents: #variables,)* } = #wrapper else { unreachable!() };
            #(#encodes;)*
        })
    }

    fn measure(&self, ctx: &MeasureContext) -> syn::Result<proc_macro2::TokenStream> {
        let wrapper = &ctx.decoded;
        let type_path = &ctx.wrapper;
        let idents = self.fields.iter().map(|(ident, _)| ident).collect::<Vec<_>>();
        let variables = idents.iter()
            .map(|ident| Ident::new(format!("variant_{}", ident).as_str(), ident.span()))
            .collect::<Vec<_>>();
        let measures = self.fields.iter().zip(variables.iter()).map(|((_, schema), variable)| {
            schema.measure(&MeasureContext {
                wrapper: quote! {},
                decoded: variable.into_token_stream(),
            })
        }).collect::<syn::Result<Vec<_>>>()?;
        Ok(quote! { {
            let #type_path { #(#idents: #variables,)* } = #wrapper else { unreachable!() };
            0 #( + #measures )*
        } })
    }

    fn measure_fixed(&self) -> syn::Result<proc_macro2::TokenStream> {
        let measures = self.fields.iter().map(|(_, schema)| {
            schema.measure_fixed()
        }).collect::<syn::Result<Vec<_>>>()?;
        Ok(quote! {
            0 #( + #measures )*
        })
    }
}

struct UnnamedFieldsSchema {
    fields: Vec<Box<dyn BinarySchema>>,
}

impl FieldsSchema for UnnamedFieldsSchema {
    fn wildcard_pattern(&self) -> proc_macro2::TokenStream {
        quote! { ( .. ) }
    }
}

impl UnnamedFieldsSchema {
    fn interpret(fields: &syn::FieldsUnnamed) -> syn::Result<UnnamedFieldsSchema> {
        let fields = fields.unnamed.iter().map(|field| {
            if let Some(ident) = &field.ident {
                return Err(syn::Error::new_spanned(
                    ident,
                    "Unnamed field must not have an identifier",
                ));
            }
            build_codec_schema(&field.attrs, Some(&field.ty))
        }).collect::<syn::Result<Vec<_>>>()?;
        Ok(UnnamedFieldsSchema { fields })
    }
}

impl BinarySchema for UnnamedFieldsSchema {
    fn decode(&self, ctx: &DecodeContext) -> syn::Result<proc_macro2::TokenStream> {
        let fields = self.fields.iter().map(|schema| {
            schema.decode(&ctx.clone())
        }).collect::<syn::Result<Vec<_>>>()?;
        Ok(quote! { ( #(#fields),* ) })
    }

    fn encode(&self, ctx: &EncodeContext) -> syn::Result<proc_macro2::TokenStream> {
        let decoded = &ctx.decoded;
        let wrapper = &ctx.wrapper;
        let variables = self.fields.iter()
            .enumerate()
            .map(|(index, _)| Ident::new(format!("variant_{}", index).as_str(), Span::call_site()))
            .collect::<Vec<_>>();
        let encodes = self.fields.iter().zip(variables.iter()).map(|(schema, variable)| {
            schema.encode(&EncodeContext {
                wrapper: quote! {},
                decoded: variable.into_token_stream(),
                encoded: ctx.encoded.clone(),
                offset: ctx.offset.clone(),
            })
        }).collect::<syn::Result<Vec<_>>>()?;
        Ok(quote! {
            let #wrapper ( #(#variables),* ) = #decoded else { unreachable!() };
            #(#encodes;)*
        })
    }

    fn measure(&self, ctx: &MeasureContext) -> syn::Result<proc_macro2::TokenStream> {
        let decoded = &ctx.decoded;
        let wrapper = &ctx.wrapper;
        let variables = self.fields.iter()
            .enumerate()
            .map(|(index, _)| Ident::new(format!("variant_{}", index).as_str(), Span::call_site()))
            .collect::<Vec<_>>();
        let measures = self.fields.iter().zip(variables.iter()).map(|(schema, variable)| {
            schema.measure(&MeasureContext {
                wrapper: quote! {},
                decoded: variable.into_token_stream(),
            })
        }).collect::<syn::Result<Vec<_>>>()?;
        Ok(quote! { {
            let #wrapper ( #(#variables),* ) = #decoded else { unreachable!() };
            0 #( + #measures )*
        } })
    }

    fn measure_fixed(&self) -> syn::Result<proc_macro2::TokenStream> {
        let measures = self.fields.iter().map(|schema| {
            schema.measure_fixed()
        }).collect::<syn::Result<Vec<_>>>()?;
        Ok(quote! {
            0 #( + #measures )*
        })
    }
}

struct UnitFieldsSchema {}

impl FieldsSchema for UnitFieldsSchema {
    fn wildcard_pattern(&self) -> proc_macro2::TokenStream {
        quote! {}
    }
}

impl BinarySchema for UnitFieldsSchema {
    fn decode(&self, _ctx: &DecodeContext) -> syn::Result<proc_macro2::TokenStream> {
        Ok(quote! {})
    }

    fn encode(&self, _ctx: &EncodeContext) -> syn::Result<proc_macro2::TokenStream> {
        Ok(quote! {})
    }

    fn measure(&self, _ctx: &MeasureContext) -> syn::Result<proc_macro2::TokenStream> {
        Ok(quote! { 0 })
    }

    fn measure_fixed(&self) -> syn::Result<proc_macro2::TokenStream> {
        Ok(quote! { 0 })
    }
}

