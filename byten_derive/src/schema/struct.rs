use syn::{Data, DeriveInput, Ident};
use quote::quote;

use super::{BinarySchema, DecodeContext, EncodeContext, MeasureContext, interpret_fields_schema};

pub fn interpret_struct_schema(input: &DeriveInput) -> syn::Result<Box<dyn BinarySchema>> {
    let Data::Struct(ref data) = input.data else {
        return Err(syn::Error::new_spanned(input, "StructSchema can only be created from struct data"));
    };
    Ok(Box::new(StructSchema {
        ident: input.ident.clone(),
        fields: interpret_fields_schema(&data.fields)?,
    }))
}

struct StructSchema {
    ident: Ident,
    fields: Box<dyn BinarySchema>,
}

impl BinarySchema for StructSchema {
    fn decode(&self, ctx: &DecodeContext) -> syn::Result<proc_macro2::TokenStream> {
        let ident = &self.ident;
        let fields = self.fields.decode(&ctx.clone())?;
        Ok(quote! { #ident #fields })
    }

    fn encode(&self, ctx: &EncodeContext) -> syn::Result<proc_macro2::TokenStream> {
        self.fields.encode(&ctx.clone())
    }

    fn measure(&self, ctx: &MeasureContext) -> syn::Result<proc_macro2::TokenStream> {
        self.fields.measure(&ctx.clone())
    }

    fn measure_fixed(&self) -> syn::Result<proc_macro2::TokenStream> {
        self.fields.measure_fixed()
    }
}

