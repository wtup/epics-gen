use quote::{quote, ToTokens};
use syn::{parse::Parse, punctuated::Punctuated, Attribute, LitStr, Token, TypePath};

pub(super) fn impl_derive_as_record(
    ast: &syn::DeriveInput,
) -> syn::Result<proc_macro2::TokenStream> {
    let id = &ast.ident;
    // Destructure fields from the ast
    let fields = if let syn::Data::Struct(syn::DataStruct {
        fields: syn::Fields::Named(syn::FieldsNamed { ref named, .. }),
        ..
    }) = ast.data
    {
        named
    } else {
        return Err(syn::Error::new_spanned(
            id,
            "Annotated code is not a struct with punctuated fields.",
        ));
    };

    let mut type_props = TypeProps::new(id.clone());

    let type_attrs: Vec<StructMeta> = get_metadata_inner("record", &ast.attrs)?;

    // occurrence check for type attrs
    for meta in type_attrs {
        match meta {
            StructMeta::RecName { kw, val } => {
                if let Some((fst_kw, _)) = type_props.type_rec_name {
                    return Err(occurrence_error(fst_kw, kw, "rec_name"));
                }
                type_props.type_rec_name = Some((kw, val));
            }
            StructMeta::RecType { kw, val } => {
                if let Some((fst_kw, _)) = type_props.type_rec_type {
                    return Err(occurrence_error(fst_kw, kw, "rec_name"));
                }
                type_props.type_rec_type = Some((kw, val));
            }
        }
    }

    for field in fields {
        let id = &field.ident;
        let syn::Field { ref attrs, .. } = field;

        let field_attrs: Vec<FieldMeta> = get_metadata_inner("record", attrs)?;
        let mut field_props = FieldProps::new(id.clone().unwrap());

        // Option 1: The field is annotated with a record and repr
        for attr in field_attrs {
            match attr {
                FieldMeta::RecName { kw, val } => {
                    if let Some((fst_kw, _)) = field_props.rec_name {
                        return Err(occurrence_error(fst_kw, kw, "rec_name"));
                    }
                    // TODO: This check is also done in the generate method, we can probably delete this
                    if let Some((fst_kw, _)) = type_props.type_rec_name {
                        return Err(rec_def_error(fst_kw, kw, "rec_name"));
                    }
                    field_props.rec_name = Some((kw, val));
                }
                FieldMeta::RecType { kw, val } => {
                    if let Some((fst_kw, _)) = field_props.rec_type {
                        return Err(occurrence_error(fst_kw, kw, "rec_type"));
                    }
                    // TODO: This check is also done in the generate method, we can probably delete this
                    if let Some((fst_kw, _)) = type_props.type_rec_type {
                        return Err(rec_def_error(fst_kw, kw, "rec_type"));
                    }
                    field_props.rec_type = Some((kw, val));
                }
                FieldMeta::RecField { kw, val } => {
                    if let Some((fst_kw, _)) = field_props.field_name {
                        return Err(occurrence_error(fst_kw, kw, "rec_field"));
                    }
                    field_props.field_name = Some((kw, val))
                }
                FieldMeta::Subst { kw, val } => {
                    if let Some((fst_kw, _)) = field_props.subst {
                        return Err(occurrence_error(fst_kw, kw, "subst"));
                    }
                    field_props.subst = Some((kw, val));
                }
                FieldMeta::Repr { kw, val } => {
                    if let Some((fst_kw, _)) = field_props.repr {
                        return Err(occurrence_error(fst_kw, kw, "repr"));
                    }
                    field_props.repr = Some((kw, val));
                }
                FieldMeta::Fmt { kw, val } => {
                    if let Some((fst_kw, _)) = field_props.format {
                        return Err(occurrence_error(fst_kw, kw, "repr"));
                    }
                    field_props.format = Some((kw, val));
                }
            }
        }
        type_props.fields.push(field_props);
    }

    let func = type_props.generate()?;
    Ok(quote!(
        impl #id {
            #func
        }
    ))
}

mod kw {
    use syn::custom_keyword;

    custom_keyword!(rec_name);
    custom_keyword!(rec_type);
    custom_keyword!(field);
    custom_keyword!(subst);
    custom_keyword!(repr);
    custom_keyword!(fmt);
}

/// Attributes that appear through the whole type
#[derive(Debug, Clone)]
struct TypeProps {
    /// struct identifier
    pub ident: syn::Ident,
    /// `rec_name` attribute appearing on the top of the type(struct)
    pub type_rec_name: Option<(kw::rec_name, LitStr)>,
    /// `rec_type` attribute appearing on the top of the type(struct)
    pub type_rec_type: Option<(kw::rec_type, LitStr)>,
    pub fields: Vec<FieldProps>,
}

impl TypeProps {
    pub fn new(ident: syn::Ident) -> Self {
        Self {
            ident,
            type_rec_name: Default::default(),
            type_rec_type: Default::default(),
            fields: Default::default(),
        }
    }

    pub fn generate(&self) -> syn::Result<proc_macro2::TokenStream> {
        match (&self.type_rec_name, &self.type_rec_type) {
            (Some((_, rname)), Some((_, rtype))) => self.generate_single_record(rname, rtype),
            (None, None) => self.generate_multiple_records(),
            (None, Some((kw, _))) => Err(syn::Error::new_spanned(
                kw,
                "global rec_type undefined".to_string(),
            )),
            (Some((kw, _)), None) => Err(syn::Error::new_spanned(
                kw,
                "global rec_type undefined".to_string(),
            )),
        }
    }

    fn generate_single_record(
        &self,
        rec_name: &LitStr,
        rec_type: &LitStr,
    ) -> syn::Result<proc_macro2::TokenStream> {
        let mut subst = quote! {};
        let mut idents: Vec<proc_macro2::TokenStream> = Vec::new();
        // Double curly braces are needed to only print the brace (without formatting). Quadruple
        // needed because this string is later again used in format! macro
        let mut record = format!(
            "record({}, \"{}\") {{{{\n",
            rec_type.value(),
            rec_name.value()
        );
        for field in &self.fields {
            // Because we're in Global record mode `field_rec_name` and field_rec_type must be
            // None
            // Handle errors if `rec_name` or `rec_type` is set
            match (&field.rec_name, &field.rec_type) {
                (None, None) => (),
                (None, Some((kw, _))) => {
                    return Err(syn::Error::new_spanned(
                        kw,
                        "rec_type cannot be set when the global rec_type exists".to_string(),
                    ));
                }
                (Some((kw, _)), None) => {
                    return Err(syn::Error::new_spanned(
                        kw,
                        "rec_name cannot be set when the global rec_type exists".to_string(),
                    ));
                }
                (Some((kw1, _)), Some((kw2, _))) => {
                    let mut err = syn::Error::new_spanned(
                        kw1,
                        "rec_name cannot be set when the global rec_type exists".to_string(),
                    );
                    err.combine(syn::Error::new_spanned(
                        kw2,
                        "rec_name cannot be set when the global rec type exists",
                    ));
                    return Err(err);
                }
            }

            let ident = &field.ident;

            // Handle `subst` attribute
            if let Some((_, val)) = &field.subst {
                let value = val.value();
                subst = quote! {
                    let res = res.replace(&#value, &self.#ident.to_string());
                };
                continue;
            }
            // Handle `repr` attribute
            let ident_repr = if let Some((_, val)) = &field.repr {
                syn::parse_str::<proc_macro2::TokenStream>(&format!(
                    "self.{} as {}",
                    ident,
                    val.path.get_ident().unwrap()
                ))
                .unwrap()
            } else {
                syn::parse_str::<proc_macro2::TokenStream>(&format!("self.{}", ident)).unwrap()
            };
            // Handle `fmt` attribute
            if let Some((_, val)) = &field.format {
                record.push_str(&val.value());
                record.push('\n');
            // Handle `field` attribute
            } else if let Some((_, val)) = &field.field_name {
                record.push_str(&format!("  field({}, \"{{}}\")\n", &val.value()));
            }
            idents.push(ident_repr);
        }
        record.push_str("}}\n");

        Ok(quote! {
            fn as_record(&self) -> String {
                let res = format!(
                    #record,
                    #(#idents,)*
                );
                #subst
                res
            }
        })
    }

    fn generate_multiple_records(&self) -> syn::Result<proc_macro2::TokenStream> {
        let mut subst = quote! {};
        let mut idents: Vec<proc_macro2::TokenStream> = Vec::new();
        let mut record = String::new();

        if self
            .fields
            .iter()
            .all(|field| field.rec_name.is_none() && field.format.is_none())
        {
            return Err(syn::Error::new_spanned(
                &self.ident,
                "type cannot be used in this context without defining `rec_name`, `rec_type` or `fmt` attributes"
                    .to_string(),
            ));
        }

        for field in &self.fields {
            let ident = &field.ident;

            // Handle `subst` attribute
            if let Some((_, val)) = &field.subst {
                let value = val.value();
                subst = quote! {
                    let res = res.replace(&#value, &self.#ident.to_string());
                };
                continue;
            }

            // Handle `repr` attribute
            let ident_repr = if let Some((_, val)) = &field.repr {
                syn::parse_str::<proc_macro2::TokenStream>(&format!(
                    "self.{} as {}",
                    ident,
                    val.path.get_ident().unwrap()
                ))
                .unwrap()
            } else {
                syn::parse_str::<proc_macro2::TokenStream>(&format!("self.{}", ident)).unwrap()
            };

            // Handle `fmt` attribute
            if let Some((_, val)) = &field.format {
                record.push_str(&val.value());
                record.push('\n');
            // Handle `field` attribute
            } else if let Some((kw, val)) = &field.field_name {
                match (&field.rec_name, &field.rec_type) {
                    (Some((_, rec_name)), Some((_, rec_type))) => {
                        record.push_str(&format!(
                            "record({}, \"{}\") {{{{\n  field({}, \"{{}}\")\n}}}}\n",
                            rec_type.value(),
                            rec_name.value(),
                            val.value()
                        ));
                    }
                    (None, None) => {
                        return Err(syn::Error::new_spanned(
                            kw,
                            "field cannot be used in this context without defining rec_name, rec_type".to_string(),
                        ));
                    }
                    (None, Some((kw_type, _))) => {
                        let mut err = syn::Error::new_spanned(
                            kw,
                            "field cannot be used in this context without defining rec_name, rec_type".to_string(),
                        );
                        err.combine(syn::Error::new_spanned(kw_type, "rec_type defined here"));
                        return Err(err);
                    }
                    (Some((kw_name, _)), None) => {
                        let mut err = syn::Error::new_spanned(
                            kw,
                            "field cannot be used in this context without defining rec_name, rec_type".to_string(),
                        );
                        err.combine(syn::Error::new_spanned(kw_name, "rec_name defined here"));
                        return Err(err);
                    }
                }
            }
            idents.push(ident_repr);
        }

        Ok(quote! {
            fn as_record(&self) -> String {
                let res = format!(
                    #record,
                    #(#idents,)*
                );
                #subst
                res
            }
        })
    }
}

#[derive(Debug, Clone)]
struct FieldProps {
    /// field identifier
    pub ident: syn::Ident,
    /// `rec_name` attribute appearing at the field(member) level
    pub rec_name: Option<(kw::rec_name, LitStr)>,
    /// `rec_type` attribute appearing at the field(member) level
    pub rec_type: Option<(kw::rec_type, LitStr)>,
    /// EPICS record field name
    pub field_name: Option<(kw::field, LitStr)>,
    /// overriding format specifier
    pub format: Option<(kw::fmt, LitStr)>,
    /// field value representation. e.g. repr = u8 will result in `format("field(DESC, {} as u8", val)`
    pub repr: Option<(kw::repr, TypePath)>,
    /// subst pattern, every record name or field can have a pattern substituted by this field
    /// member
    pub subst: Option<(kw::subst, LitStr)>,
}

impl FieldProps {
    fn new(ident: syn::Ident) -> Self {
        Self {
            ident,
            rec_name: Default::default(),
            rec_type: Default::default(),
            field_name: Default::default(),
            format: Default::default(),
            repr: Default::default(),
            subst: Default::default(),
        }
    }
}
// Copied from `strum_macros`. It uses the internal syn::Parse method to parse into a custom type <T>. check `syn::parse` module
// documentation for more details.
//
/// Parses a collection of Attributes and collects it into a collection of a user defined type T.
fn get_metadata_inner<'a, T: Parse>(
    ident: &str,
    it: impl IntoIterator<Item = &'a Attribute>,
) -> syn::Result<Vec<T>> {
    it.into_iter()
        .filter(|attr| attr.path().is_ident(ident))
        .try_fold(Vec::new(), |mut vec, attr| {
            vec.extend(attr.parse_args_with(Punctuated::<T, Token![,]>::parse_terminated)?);
            Ok(vec)
        })
}

#[derive(Debug, Clone)]
enum StructMeta {
    RecName { kw: kw::rec_name, val: syn::LitStr },
    RecType { kw: kw::rec_type, val: syn::LitStr },
}

impl Parse for StructMeta {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        //TODO: use match to enforce handling all variants
        if lookahead.peek(kw::rec_name) {
            let kw = input.parse()?;
            let _: Token![=] = input.parse()?;
            let val = input.parse()?;
            Ok(StructMeta::RecName { kw, val })
        } else if lookahead.peek(kw::rec_type) {
            let kw = input.parse()?;
            let _: Token![=] = input.parse()?;
            let val = input.parse()?;
            Ok(StructMeta::RecType { kw, val })
        } else {
            Err(lookahead.error())
        }
    }
}

#[derive(Debug, Clone)]
enum FieldMeta {
    RecName { kw: kw::rec_name, val: syn::LitStr },
    RecType { kw: kw::rec_type, val: syn::LitStr },
    RecField { kw: kw::field, val: syn::LitStr },
    Repr { kw: kw::repr, val: syn::TypePath },
    Fmt { kw: kw::fmt, val: syn::LitStr },
    Subst { kw: kw::subst, val: syn::LitStr },
}

impl Parse for FieldMeta {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        //TODO: use match to enforce handling all variants
        if lookahead.peek(kw::rec_name) {
            let kw = input.parse()?;
            let _: Token![=] = input.parse()?;
            let val = input.parse()?;
            Ok(FieldMeta::RecName { kw, val })
        } else if lookahead.peek(kw::rec_type) {
            let kw = input.parse()?;
            let _: Token![=] = input.parse()?;
            let val = input.parse()?;
            Ok(FieldMeta::RecType { kw, val })
        } else if lookahead.peek(kw::subst) {
            let kw = input.parse()?;
            let _: Token![=] = input.parse()?;
            let val = input.parse()?;
            Ok(FieldMeta::Subst { kw, val })
        } else if lookahead.peek(kw::field) {
            let kw = input.parse()?;
            let _: Token![=] = input.parse()?;
            let val = input.parse()?;
            Ok(FieldMeta::RecField { kw, val })
        } else if lookahead.peek(kw::repr) {
            let kw = input.parse()?;
            let _: Token![=] = input.parse()?;
            let val = input.parse()?;
            Ok(FieldMeta::Repr { kw, val })
        } else if lookahead.peek(kw::fmt) {
            let kw = input.parse()?;
            let _: Token![=] = input.parse()?;
            let val = input.parse()?;
            Ok(FieldMeta::Fmt { kw, val })
        } else {
            Err(lookahead.error())
        }
    }
}

pub fn occurrence_error<T: ToTokens>(fst: T, snd: T, attr: &str) -> syn::Error {
    let mut e = syn::Error::new_spanned(
        snd,
        format!("Found multiple occurrences of strum({})", attr),
    );
    e.combine(syn::Error::new_spanned(fst, "first one here"));
    e
}

pub fn rec_def_error<T: ToTokens>(fst: T, snd: T, attr: &str) -> syn::Error {
    let mut e = syn::Error::new_spanned(
        snd,
        format!(
            "Global `{}` attribute defined, overriding is not possible",
            attr
        ),
    );
    e.combine(syn::Error::new_spanned(fst, "first one here"));
    e
}
