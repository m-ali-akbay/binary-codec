use quote::quote;
use syn::{Data, DeriveInput, Expr, Ident, Meta, Type, spanned::Spanned as _};

use super::{
    BinarySchema, DecodeContext, EncodeContext, FieldsSchema, MeasureContext, build_codec_schema,
    interpret_fields_schema,
};

pub fn interpret_enum_schema(input: &DeriveInput) -> syn::Result<Box<dyn BinarySchema>> {
    let Data::Enum(ref data) = input.data else {
        return Err(syn::Error::new(
            input.span(),
            "EnumSchema can only be created from enum data",
        ));
    };

    let repr = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("repr"))
        .ok_or_else(|| syn::Error::new(input.span(), "Enum must have a repr attribute"))?;
    let repr = match &repr.meta {
        Meta::List(meta) => meta.parse_args::<Type>()?,
        _ => {
            return Err(syn::Error::new(
                repr.span(),
                "Invalid repr attribute format",
            ));
        }
    };

    let discriminator = build_codec_schema(&input.attrs, Some(&repr))?;

    let variants = data
        .variants
        .iter()
        .map(|variant| {
            let ident = variant.ident.clone();
            let schema = interpret_fields_schema(&variant.fields)?;
            let discriminant = match &variant.discriminant {
                Some((_, expr)) => expr.clone(),
                None => {
                    return Err(syn::Error::new(
                        variant.span(),
                        "Enum variants must have discriminants",
                    ));
                }
            };
            Ok((ident, schema, discriminant))
        })
        .collect::<syn::Result<_>>()?;
    Ok(Box::new(EnumSchema {
        ident: input.ident.clone(),
        discriminator,
        variants,
    }))
}

struct EnumSchema {
    ident: Ident,
    discriminator: Box<dyn BinarySchema>,
    variants: Vec<(Ident, Box<dyn FieldsSchema>, Expr)>,
}

impl BinarySchema for EnumSchema {
    fn decode(&self, ctx: &DecodeContext) -> syn::Result<proc_macro2::TokenStream> {
        let ident = &self.ident;
        let variants = self
            .variants
            .iter()
            .map(|(variant_ident, schema, discriminant)| {
                let decode = schema.decode(&ctx.clone())?;
                Ok(quote! {
                    #discriminant => {
                        Ok(#ident::#variant_ident #decode)
                    }
                })
            })
            .collect::<syn::Result<Vec<_>>>()?;
        let encoded = &ctx.encoded;
        let offset = &ctx.offset;
        let decode_discriminant = self.discriminator.decode(&DecodeContext {
            encoded: encoded.clone(),
            offset: offset.clone(),
        })?;
        Ok(quote! { {
            let discriminant = #decode_discriminant;
            match discriminant {
                #(#variants),*,
                _ => Err(::byten::DecodeError::InvalidDiscriminant),
            }?
        } })
    }

    fn encode(&self, ctx: &EncodeContext) -> syn::Result<proc_macro2::TokenStream> {
        let ident = &self.ident;
        let decoded = ctx.decoded.clone();
        let encoded = ctx.encoded.clone();
        let offset = ctx.offset.clone();
        let variants = self
            .variants
            .iter()
            .map(|(variant_ident, schema, discriminant)| {
                let encoder_discriminant = self.discriminator.encode(&EncodeContext {
                    wrapper: quote! {},
                    decoded: quote! { (&#discriminant) },
                    encoded: encoded.clone(),
                    offset: offset.clone(),
                })?;
                let encode = schema.encode(&EncodeContext {
                    wrapper: quote! { #ident::#variant_ident },
                    decoded: quote! { variant },
                    encoded: encoded.clone(),
                    offset: offset.clone(),
                })?;
                let wildcard_pattern = schema.wildcard_pattern();
                Ok(quote! {
                    variant @ #ident::#variant_ident #wildcard_pattern => {
                        #encoder_discriminant;
                        #encode
                    }
                })
            })
            .collect::<syn::Result<Vec<_>>>()?;
        Ok(quote! {
            match #decoded {
                #(#variants),*
            }
        })
    }

    fn measure(&self, ctx: &MeasureContext) -> syn::Result<proc_macro2::TokenStream> {
        let ident = &self.ident;
        let decoded = ctx.decoded.clone();
        let variants = self
            .variants
            .iter()
            .map(|(variant_ident, schema, discriminant)| {
                let measure_discriminant = self.discriminator.measure(&MeasureContext {
                    wrapper: quote! {},
                    decoded: quote! { (&#discriminant) },
                })?;

                let measure = schema.measure(&MeasureContext {
                    wrapper: quote! { #ident::#variant_ident },
                    decoded: quote! { variant },
                })?;
                let wildcard_pattern = schema.wildcard_pattern();
                Ok(quote! {
                    variant @ #ident::#variant_ident #wildcard_pattern => {
                        #measure_discriminant + #measure
                    }
                })
            })
            .collect::<syn::Result<Vec<_>>>()?;
        Ok(quote! {
            match #decoded {
                #(#variants),*
            }
        })
    }

    fn measure_fixed(&self) -> syn::Result<proc_macro2::TokenStream> {
        Err(syn::Error::new(
            self.ident.span(),
            "Fixed measure is not yet supported for enums",
        ))
    }
}
