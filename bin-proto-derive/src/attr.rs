use proc_macro2::{Span, TokenStream};
use syn::{parse::Parser, punctuated::Punctuated, spanned::Spanned, token::Add, Error, Result};

#[derive(Debug, Default)]
pub struct Attrs {
    pub discriminant_type: Option<syn::Type>,
    pub discriminant: Option<syn::Expr>,
    pub ctx: Option<syn::Type>,
    pub ctx_bounds: Option<Punctuated<syn::TypeParamBound, Add>>,
    pub write_value: Option<syn::Expr>,
    pub bits: Option<u32>,
    pub flexible_array_member: bool,
    pub length: Option<syn::Expr>,
    pub condition: Option<syn::Expr>,
}

impl Attrs {
    pub fn validate_enum(&self, span: Span) -> Result<()> {
        if self.discriminant_type.is_none() {
            return Err(Error::new(
                span,
                "expected discriminant_type attribute for enum",
            ));
        }
        if self.discriminant.is_some() {
            return Err(Error::new(
                span,
                "unexpected discriminant attribute for enum",
            ));
        }
        if self.ctx.is_some() && self.ctx_bounds.is_some() {
            return Err(Error::new(
                span,
                "cannot specify ctx and ctx_bounds simultaneously",
            ));
        }
        if self.write_value.is_some() {
            return Err(Error::new(
                span,
                "unexpected write_value attribute for enum",
            ));
        }
        if self.flexible_array_member {
            return Err(Error::new(
                span,
                "unexpected flexible_array_member attribute for enum",
            ));
        }
        if self.length.is_some() {
            return Err(Error::new(span, "unexpected length attribute for enum"));
        }
        if self.condition.is_some() {
            return Err(Error::new(span, "unexpected condition attribute for enum"));
        }
        Ok(())
    }

    pub fn validate_variant(&self, span: Span) -> Result<()> {
        if self.discriminant_type.is_some() {
            return Err(Error::new(
                span,
                "unexpected discriminant_type attribute for variant",
            ));
        }
        if self.ctx.is_some() {
            return Err(Error::new(span, "unexpected ctx attribute for variant"));
        }
        if self.ctx_bounds.is_some() {
            return Err(Error::new(
                span,
                "unexpected ctx_bounds attribute for variant",
            ));
        }
        if self.write_value.is_some() {
            return Err(Error::new(
                span,
                "unexpected write_value attribute for variant",
            ));
        }
        if self.bits.is_some() {
            return Err(Error::new(span, "unexpected bits attribute for variant"));
        }
        if self.flexible_array_member {
            return Err(Error::new(
                span,
                "unexpected flexible_array_member attribute for variant",
            ));
        }
        if self.length.is_some() {
            return Err(Error::new(span, "unexpected length attribute for variant"));
        }
        if self.condition.is_some() {
            return Err(Error::new(span, "unexpected condition attribute for variant"));
        }
        Ok(())
    }

    pub fn validate_field(&self, span: Span) -> Result<()> {
        if self.discriminant_type.is_some() {
            return Err(Error::new(
                span,
                "unexpected discriminant_type attribute for field",
            ));
        }
        if self.discriminant.is_some() {
            return Err(Error::new(
                span,
                "unexpected discriminant attribute for field",
            ));
        }
        if self.ctx.is_some() {
            return Err(Error::new(span, "unexpected ctx attribute for variant"));
        }
        if self.ctx_bounds.is_some() {
            return Err(Error::new(
                span,
                "unexpected ctx_bounds attribute for variant",
            ));
        }
        if [
            self.bits.is_some(),
            self.flexible_array_member,
            self.length.is_some(),
        ]
        .iter()
        .filter(|b| **b)
        .count()
            > 1
        {
            return Err(Error::new(
                span,
                "bits, flexible_array_member, and length are mutually-exclusive attributes",
            ));
        }
        Ok(())
    }

    pub fn ctx_ty(&self) -> TokenStream {
        self.ctx
            .as_ref()
            .map(|ctx| quote!(#ctx))
            .unwrap_or(quote!(__Ctx))
    }
}

impl TryFrom<&[syn::Attribute]> for Attrs {
    type Error = syn::Error;

    fn try_from(value: &[syn::Attribute]) -> Result<Self> {
        let meta_lists = value.iter().filter_map(|attr| match attr.parse_meta() {
            Ok(syn::Meta::List(meta_list)) => {
                if meta_list.path.get_ident()
                    == Some(&syn::Ident::new("protocol", Span::call_site()))
                {
                    Some(meta_list)
                } else {
                    None
                }
            }
            _ => None,
        });

        let mut attribs = Attrs::default();
        for meta_list in meta_lists {
            for meta in meta_list.nested.iter() {
                match meta {
                    syn::NestedMeta::Meta(syn::Meta::NameValue(name_value)) => match name_value
                        .path
                        .get_ident()
                    {
                        Some(ident) => match ident.to_string().as_str() {
                            "discriminant_type" => {
                                attribs.discriminant_type =
                                    Some(meta_name_value_to_parse(name_value)?)
                            }
                            "discriminant" => {
                                attribs.discriminant = Some(meta_name_value_to_parse(name_value)?)
                            }
                            "ctx" => attribs.ctx = Some(meta_name_value_to_parse(name_value)?),
                            "ctx_bounds" => {
                                attribs.ctx_bounds =
                                    Some(meta_name_value_to_punctuated(name_value)?)
                            }
                            "bits" => attribs.bits = Some(meta_name_value_to_u32(name_value)?),
                            "write_value" => {
                                attribs.write_value = Some(meta_name_value_to_parse(name_value)?)
                            }
                            "length" => {
                                attribs.length = Some(meta_name_value_to_parse(name_value)?)
                            }
                            "condition" => {
                                attribs.condition = Some(meta_name_value_to_parse(name_value)?)
                            }
                            _ => {
                                return Err(Error::new(meta_list.span(), "unrecognised attribute"))
                            }
                        },
                        None => {
                            return Err(Error::new(meta_list.span(), "failed to parse attribute"))
                        }
                    },
                    syn::NestedMeta::Meta(syn::Meta::Path(path)) => match path.get_ident() {
                        Some(ident) => match ident.to_string().as_str() {
                            "flexible_array_member" => attribs.flexible_array_member = true,
                            _ => {
                                return Err(Error::new(meta_list.span(), "unrecognised attribute"))
                            }
                        },
                        None => {
                            return Err(Error::new(path.get_ident().span(), "expected identifier"))
                        }
                    },
                    _ => return Err(Error::new(meta_list.span(), "unrecognised attribute")),
                };
            }
        }
        Ok(attribs)
    }
}

fn meta_name_value_to_parse<T: syn::parse::Parse>(name_value: &syn::MetaNameValue) -> Result<T> {
    match name_value.lit {
        syn::Lit::Str(ref s) => syn::parse_str::<T>(s.value().as_str())
            .map_err(|_| Error::new(name_value.span(), "Failed to parse")),

        _ => Err(Error::new(name_value.span(), "Expected string")),
    }
}

fn meta_name_value_to_u32(name_value: &syn::MetaNameValue) -> Result<u32> {
    match name_value.lit {
        syn::Lit::Int(ref i) => i
            .base10_parse()
            .map_err(|_| Error::new(name_value.span(), "Failed to parse u32")),
        _ => Err(Error::new(name_value.span(), "Expected integer")),
    }
}

fn meta_name_value_to_punctuated<T: syn::parse::Parse, P: syn::parse::Parse>(
    name_value: &syn::MetaNameValue,
) -> Result<Punctuated<T, P>> {
    match name_value.lit {
        syn::Lit::Str(ref s) => Punctuated::parse_terminated
            .parse_str(s.value().as_str())
            .map_err(|_| Error::new(name_value.span(), "Failed to parse")),
        _ => Err(Error::new(name_value.span(), "Expected string")),
    }
}
