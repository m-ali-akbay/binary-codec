use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
    Attribute, Expr, Ident, Meta, Token, Type,
    parse::{ParseStream, Parser},
    parse_quote,
    spanned::Spanned,
    token::Brace,
};

use super::{BinarySchema, DecodeContext, EncodeContext, MeasureContext};

struct CodecSchema {
    expr: Expr,
}

impl BinarySchema for CodecSchema {
    fn decode(&self, ctx: &DecodeContext) -> syn::Result<proc_macro2::TokenStream> {
        let expr = &self.expr;
        let encoded = &ctx.encoded;
        let offset = &ctx.offset;
        Ok(quote! { ::byten::Decoder::decode(&#expr, #encoded, #offset)? })
    }

    fn encode(&self, ctx: &EncodeContext) -> syn::Result<proc_macro2::TokenStream> {
        let expr = &self.expr;
        let decoded = &ctx.decoded;
        let encoded = &ctx.encoded;
        let offset = &ctx.offset;
        Ok(quote! { ::byten::Encoder::encode(&#expr, #decoded, #encoded, #offset)? })
    }

    fn measure_fixed(&self) -> syn::Result<proc_macro2::TokenStream> {
        let expr = &self.expr;
        Ok(quote! { ::byten::FixedMeasurer::measure_fixed(&#expr) })
    }

    fn measure(&self, ctx: &MeasureContext) -> syn::Result<proc_macro2::TokenStream> {
        let expr = &self.expr;
        let decoded = &ctx.decoded;
        Ok(quote! { ::byten::Measurer::measure(&#expr, #decoded)? })
    }
}

trait Operand {
    fn span(&self) -> Span;
    fn codec(&self) -> Expr;
    fn typ(&self) -> syn::Result<Type> {
        Err(syn::Error::new(
            self.span(),
            "Operand does not support type extraction",
        ))
    }
}

struct DefaultOperand {
    span: Span,
}

impl Operand for DefaultOperand {
    fn span(&self) -> Span {
        self.span
    }

    fn codec(&self) -> Expr {
        parse_quote! { ::byten::DefaultCodec::default_codec() }
    }
}

struct TypeOperand {
    span: Span,
    typ: Type,
}

impl Operand for TypeOperand {
    fn span(&self) -> Span {
        self.span
    }

    fn codec(&self) -> Expr {
        let ty = &self.typ;
        parse_quote! { <#ty as ::byten::DefaultCodec>::default_codec() }
    }

    fn typ(&self) -> syn::Result<Type> {
        Ok(self.typ.clone())
    }
}

enum EndianOperand {
    Big(Span, Type),
    Little(Span, Type),
}

impl Operand for EndianOperand {
    fn span(&self) -> Span {
        match self {
            EndianOperand::Big(span, _) => *span,
            EndianOperand::Little(span, _) => *span,
        }
    }

    fn codec(&self) -> Expr {
        match self {
            EndianOperand::Big(_, ty) => {
                parse_quote! { ::byten::EndianCodec::<#ty>::new(::byten::Endianness::Big) }
            }
            EndianOperand::Little(_, ty) => {
                parse_quote! { ::byten::EndianCodec::<#ty>::new(::byten::Endianness::Little) }
            }
        }
    }
}

struct VecOperand {
    span: Span,
    item: Box<dyn Operand>,
    length: Box<dyn Operand>,
}

impl Operand for VecOperand {
    fn span(&self) -> Span {
        self.span
    }

    fn codec(&self) -> Expr {
        let item_codec = self.item.codec();
        let length_codec = self.length.codec();
        parse_quote! { ::byten::VecCodec::new(#item_codec, #length_codec) }
    }
}

struct OwnOperand {
    span: Span,
    base: Box<dyn Operand>,
}

impl Operand for OwnOperand {
    fn span(&self) -> Span {
        self.span
    }

    fn codec(&self) -> Expr {
        let base = self.base.codec();
        parse_quote! { ::byten::OwnedCodec::new(#base) }
    }
}

struct OptOperand {
    span: Span,
    base: Box<dyn Operand>,
}

impl Operand for OptOperand {
    fn span(&self) -> Span {
        self.span
    }

    fn codec(&self) -> Expr {
        let base = self.base.codec();
        parse_quote! { ::byten::OptionCodec::new(#base) }
    }
}

struct UTF8Operand {
    span: Span,
    base: Box<dyn Operand>,
}

impl Operand for UTF8Operand {
    fn span(&self) -> Span {
        self.span
    }

    fn codec(&self) -> Expr {
        let base = self.base.codec();
        parse_quote! { ::byten::UTF8Codec::new(#base) }
    }
}

struct CodecOperand {
    span: Span,
    codec: Expr,
}

impl Operand for CodecOperand {
    fn span(&self) -> Span {
        self.span
    }

    fn codec(&self) -> Expr {
        self.codec.clone()
    }
}

struct UVarBEOperand {
    span: Span,
    typ: Type,
}

impl Operand for UVarBEOperand {
    fn span(&self) -> Span {
        self.span
    }

    fn codec(&self) -> Expr {
        let ty = &self.typ;
        parse_quote! { ::byten::UVarBECodec::<#ty>::new() }
    }
}

struct BytesOperand {
    span: Span,
    length: Box<dyn Operand>,
}

impl Operand for BytesOperand {
    fn span(&self) -> Span {
        self.span
    }

    fn codec(&self) -> Expr {
        let length_codec = self.length.codec();
        parse_quote! { ::byten::BytesCodec::new(#length_codec) }
    }
}

struct RemainingOperand {
    span: Span,
}

impl Operand for RemainingOperand {
    fn span(&self) -> Span {
        self.span
    }
    fn codec(&self) -> Expr {
        parse_quote! { ::byten::RemainingCodec::new() }
    }
}

struct ArrOperand {
    span: Span,
    item: Box<dyn Operand>,
}

impl Operand for ArrOperand {
    fn span(&self) -> Span {
        self.span
    }

    fn codec(&self) -> Expr {
        let item_codec = self.item.codec();
        parse_quote! { ::byten::ArrayCodec::new(#item_codec) }
    }
}

pub fn build_codec_expr(tokens: TokenStream) -> syn::Result<Expr> {
    let span = tokens.span();
    let operand = Parser::parse2(
        move |stream: ParseStream<'_>| {
            build_codec_pipeline(stream, Box::new(DefaultOperand { span }))
        },
        tokens,
    )?;

    Ok(operand.codec())
}

pub fn build_codec_schema(
    attr: &Vec<Attribute>,
    typ: Option<&Type>,
) -> syn::Result<Box<dyn BinarySchema>> {
    let mut operand: Box<dyn Operand> = match typ {
        Some(typ) => Box::new(TypeOperand {
            span: typ.span(),
            typ: typ.clone(),
        }),
        None => Box::new(DefaultOperand {
            span: Span::call_site(),
        }),
    };

    for attribute in attr {
        if !attribute.path().is_ident("byten") {
            continue;
        }
        match &attribute.meta {
            Meta::List(meta) => {
                let tokens = meta.tokens.clone();
                operand = Parser::parse2(
                    move |stream: ParseStream<'_>| build_codec_pipeline(stream, operand),
                    tokens,
                )?;
            }
            _ => {
                return Err(syn::Error::new(
                    attribute.span(),
                    "Invalid byten attribute format",
                ));
            }
        }
    }

    Ok(Box::new(CodecSchema {
        expr: operand.codec(),
    }))
}

fn build_codec_pipeline(
    stream: ParseStream,
    mut operand: Box<dyn Operand>,
) -> Result<Box<dyn Operand>, syn::Error> {
    loop {
        operand = build_codec(stream, operand)?;
        if stream.is_empty() {
            break Ok(operand);
        }
    }
}

fn build_codec(
    stream: ParseStream,
    operand: Box<dyn Operand>,
) -> Result<Box<dyn Operand>, syn::Error> {
    if stream.peek(Brace) {
        let content;
        syn::braced!(content in stream);
        let expr: Expr = content.parse()?;
        return Ok(Box::new(CodecOperand {
            span: content.span(),
            codec: expr,
        }));
    }

    if stream.peek(Token![..]) {
        let _dots: Token![..] = stream.parse()?;
        return Ok(Box::new(RemainingOperand { span: _dots.span() }));
    }

    if stream.peek(Token![$]) {
        let _dollar: Token![$] = stream.parse()?;
        let ident: Ident = stream.parse()?;
        return match ident.to_string().as_str() {
            "own" => Ok(Box::new(OwnOperand {
                span: ident.span(),
                base: operand,
            })),
            "be" => Ok(Box::new(EndianOperand::Big(ident.span(), operand.typ()?))),
            "le" => Ok(Box::new(EndianOperand::Little(
                ident.span(),
                operand.typ()?,
            ))),
            "uvarbe" => Ok(Box::new(UVarBEOperand {
                span: ident.span(),
                typ: operand.typ()?,
            })),
            "bytes" => {
                let content;
                syn::bracketed!(content in stream);
                let length = build_codec_pipeline(
                    &content,
                    Box::new(DefaultOperand { span: ident.span() }),
                )?;
                Ok(Box::new(BytesOperand {
                    span: ident.span(),
                    length,
                }))
            }
            "vec" => {
                let content;
                syn::parenthesized!(content in stream);
                let item = build_codec_pipeline(
                    &content,
                    Box::new(DefaultOperand { span: ident.span() }),
                )?;

                let content;
                syn::bracketed!(content in stream);
                let length = build_codec_pipeline(
                    &content,
                    Box::new(DefaultOperand { span: ident.span() }),
                )?;

                Ok(Box::new(VecOperand {
                    span: ident.span(),
                    item,
                    length,
                }))
            }
            "arr" => {
                let content;
                syn::bracketed!(content in stream);
                let item = build_codec_pipeline(
                    &content,
                    Box::new(DefaultOperand { span: ident.span() }),
                )?;
                Ok(Box::new(ArrOperand {
                    span: ident.span(),
                    item,
                }))
            }
            "opt" => Ok(Box::new(OptOperand {
                span: ident.span(),
                base: operand,
            })),
            "utf8" => Ok(Box::new(UTF8Operand {
                span: ident.span(),
                base: operand,
            })),
            _ => Err(syn::Error::new(
                ident.span(),
                format!("Unknown codec modifier: {}", ident),
            )),
        };
    }

    let typ: Type = stream.parse()?;
    Ok(Box::new(TypeOperand {
        span: stream.span(),
        typ,
    }))
}
