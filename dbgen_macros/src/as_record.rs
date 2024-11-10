use quote::quote;
use syn::{parenthesized, parse::Parse, punctuated::Punctuated, Attribute, LitStr, Token};

mod kw {
    use syn::custom_keyword;

    custom_keyword!(record);
    custom_keyword!(subst);
    custom_keyword!(repr);
}

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
        unimplemented!();
    };
    let mut fun_defs: Vec<proc_macro2::TokenStream> = Vec::new();
    let mut fun_names = Vec::new();
    let mut subst_tst = proc_macro2::TokenStream::new();
    let mut members = Vec::new();
    let mut member_type = Vec::new();

    for field in fields.iter() {
        let id = &field.ident;
        let ty = &field.ty;
        // Get record string from attribute
        members.push(id);
        member_type.push(ty);

        let syn::Field { ref attrs, .. } = field;

        //TODO: get rid of the unwrap!
        let parsed_attrs: Vec<VariantMeta> = get_metadata_inner("dbgen", attrs)?;

        // Option 1: The field is annotated with a record and repr
        parsed_attrs.into_iter().for_each(|attr| {
            let fun_name;

            match attr {
                VariantMeta::Record { record, repr, .. } => {
                    // If the attribute is a `record` we need to define a method named
                    //<field_name>_as_record().
                    fun_name = syn::parse_str::<proc_macro2::Ident>(&format!(
                        "{}_as_record",
                        id.clone().unwrap()
                    ))
                    .unwrap();

                    // Push the function name into the collection for later printing.
                    fun_names.push(fun_name.clone());

                    // If repr attr is present, it will format self as the repr type, else use
                    // default format.
                    let field_val = if let Some(repr_ty) = repr {
                        syn::parse_str::<proc_macro2::TokenStream>(&format!(
                            "self.{} as {}",
                            id.clone().unwrap(),
                            repr_ty.path.get_ident().unwrap()
                        ))
                        .unwrap()
                    } else {
                        syn::parse_str::<proc_macro2::TokenStream>(&format!(
                            "self.{}",
                            id.clone().unwrap()
                        ))
                        .unwrap()
                    };

                    let field_as_record_fn = quote! {
                        fn #fun_name(&self) -> std::string::String {
                            format!(#record, #field_val)
                        }
                    };

                    fun_defs.push(field_as_record_fn);
                }
                VariantMeta::Subst { subst, .. } => {
                    subst_tst = quote!(
                        let res = res.replace(#subst, &format!("{}", self.#id));
                    );
                }
            }
        });
    }

    let res = quote! {
        impl #id {
            fn as_record(&self) -> String {
                let mut res = std::string::String::new();
                #(
                    res.push_str(&self.#fun_names());
                    res.push_str("\n");
                )*
                #subst_tst
                res
            }
            #(#fun_defs)*
        }
    };
    Ok(res)
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
enum VariantMeta {
    Record {
        kw: kw::record,
        record: LitStr,
        repr: Option<syn::TypePath>,
    },
    Subst {
        kw: kw::subst,
        subst: LitStr,
    },
}

impl Parse for VariantMeta {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::record) {
            let mut repr = None;
            let kw = input.parse()?;
            let _: Token![=] = input.parse()?;
            let record = input.parse()?;
            //Checks if repr attribute is set
            let lookahead_inner = input.lookahead1();
            if lookahead_inner.peek(Token![,]) {
                let _: Token![,] = input.parse()?;
                let _: kw::repr = input.parse().map_err(|mut e| {
                    e.combine(syn::Error::new(
                        e.span(),
                        "When specifying record attribute, \
                    it can optionally be followed by the repr attribute, \
                    which tells the macro how the value is \
                    printed (similar to a format specifier). \
                    e.g. repr = u8, prints the value as an u8",
                    ));
                    e
                })?;
                let content;
                let _ = parenthesized!(content in input);
                repr = Some(content.parse()?);
            }
            Ok(VariantMeta::Record { kw, record, repr })
        } else if lookahead.peek(kw::subst) {
            let kw = input.parse()?;
            let _: Token![=] = input.parse()?;
            let subst = input.parse()?;
            Ok(VariantMeta::Subst { kw, subst })
        } else {
            Err(lookahead.error())
        }
    }
}
