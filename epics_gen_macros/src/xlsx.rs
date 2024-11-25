use quote::quote;

pub(super) fn impl_derive_from_xstring(ast: &syn::DeriveInput) -> proc_macro2::TokenStream {
    let id = &ast.ident;
    let res = quote! {
        impl TryFrom<epics_gen::XlsxData> for #id {
            type Error = epics_gen::ParseErrorKind;

            fn try_from(value: epics_gen::XlsxData) -> Result<Self, Self::Error> {
                value
                    .get_string()
                    .ok_or_else(|| Self::Error::ValueMissing)?
                    .try_into()
                    .map_err(|_| Self::Error::InvalidValue)
            }
        }
    };
    res
}

pub(super) fn impl_derive_from_xfloat(
    ast: &syn::DeriveInput,
) -> syn::Result<proc_macro2::TokenStream> {
    let id = &ast.ident;
    let res = quote! {
        impl TryFrom<epics_gen::XlsxData> for #id {
            type Error = epics_gen::ParseErrorKind;

            fn try_from(value: epics_gen::XlsxData) -> Result<Self, Self::Error> {
                value
                    .get_float()
                    .ok_or_else(|| Self::Error::ValueMissing)?
                    .try_into()
                    .map_err(|_| Self::Error::InvalidValue)
            }
        }
    };
    Ok(res)
}

pub(super) fn impl_derive_xlsx_row(
    ast: &syn::DeriveInput,
) -> syn::Result<proc_macro2::TokenStream> {
    let id = &ast.ident;
    //Iterate through all the fields and try to convert them into types and push them into the
    //struct
    let fields = if let syn::Data::Struct(syn::DataStruct {
        fields: syn::Fields::Named(syn::FieldsNamed { ref named, .. }),
        ..
    }) = ast.data
    {
        named
    } else {
        return Err(syn::Error::new_spanned(
            ast,
            "Cannot implement for struct without fields.",
        ));
    };
    let mut field_convert = Vec::new();

    for (i, field) in fields.iter().enumerate() {
        let id = &field.ident;
        let ty = &field.ty;

        let mut single_element = quote! {
            {
                let val = row.pop().unwrap();
                val.clone().try_into().map_err(|kind| epics_gen::ParseError::new_in_table(kind,epics_gen::XlsxCell::new((row_num as u32, #i as u32), val), table_name.to_owned()))?
            }
        };

        let (type_len, ty): (usize, &syn::Type) = match ty {
            syn::Type::Array(syn::TypeArray { elem, .. }) => {
                let (type_len, _) = destructure_array(ty)?;
                (type_len.base10_parse().unwrap(), elem)
            }
            _ => (1, ty),
        };

        if extern_type_is(ty, "Option") {
            let inner_type = extract_generic_type(ty)?;
            single_element = quote! {
                {
                    let val = row.pop().unwrap();
                    match #inner_type::try_from(val.clone()) {
                        Err(epics_gen::ParseErrorKind::ValueMissing) => None,
                        v => Some(v.map_err(|kind| epics_gen::ParseError::new_in_table(kind,epics_gen::XlsxCell::new((row_num as u32, #i as u32), val), table_name.to_owned()))?),
                    }
                }
            };
            inner_type
        } else {
            ty
        };

        let field_output = if type_len > 1 {
            let mut elements = Vec::new();
            for _ in 0..type_len {
                elements.push(single_element.clone());
            }
            quote! {
                #id: [
                    #(#elements,)*
                ]
            }
        } else {
            quote! {
                #id: {
                    #single_element
                }
            }
        };

        field_convert.push(field_output);
    }
    let res = quote! {
        impl epics_gen::FromXlsxRow for #id
        where Self: Sized {
            type Error = epics_gen::ParseError;
            fn from_xlsx_row(row: epics_gen::XlsxRow, row_num: usize, table_name:&str)
            -> ::std::result::Result<Self, epics_gen::ParseError> {
                let mut row = row.clone();
                row.reverse();
                Ok(Self {
                    #(#field_convert,)*
                })
            }

        }
    };
    Ok(res)
}

fn destructure_array(ty: &syn::Type) -> syn::Result<(syn::LitInt, syn::Ident)> {
    if let syn::Type::Array(syn::TypeArray { elem, len, .. }) = ty {
        if let syn::Expr::Lit(syn::ExprLit { lit, .. }) = len {
            let lit_int = if let syn::Lit::Int(lit_int) = lit {
                lit_int
            } else {
                return Err(syn::Error::new_spanned(lit, "Len is not an integer"));
            };

            let path = match ungroup(elem) {
                syn::Type::Path(ty) => &ty.path,
                _ => return Err(syn::Error::new_spanned(elem, "Not a path")),
            };
            let seg = match path.segments.last() {
                Some(seg) => seg,
                None => return Err(syn::Error::new_spanned(path, "No segments in path")),
            };
            Ok((lit_int.clone(), seg.ident.clone()))
        } else {
            Err(syn::Error::new_spanned(len, "Len is not an literal!"))
        }
    } else {
        Err(syn::Error::new_spanned(ty, "Type is not an array!"))
    }
}

// If member type is vector or array check for attribute named `len`. This tells the macro how many
// elements are deserialized into the structure. If no `len` macro is used, just push all elements
// until the row ends.

fn extern_type_is<'a>(ty: &'a syn::Type, pattern: &'a str) -> bool {
    let path = match ungroup(ty) {
        syn::Type::Path(ty) => &ty.path,
        _ => {
            return false;
        }
    };
    let seg = match path.segments.last() {
        Some(seg) => seg,
        None => {
            return false;
        }
    };
    let args = match &seg.arguments {
        syn::PathArguments::AngleBracketed(bracketed) => &bracketed.args,
        _ => {
            return false;
        }
    };
    if seg.ident == pattern && args.len() == 1 {
        matches!(&args[0], syn::GenericArgument::Type(_))
    } else {
        false
    }
}

fn extract_generic_type(ty: &syn::Type) -> syn::Result<&syn::Type> {
    let path = match ungroup(ty) {
        syn::Type::Path(ty) => &ty.path,
        _ => {
            return Err(syn::Error::new_spanned(ty, "Could not ungroup type!"));
        }
    };
    let seg = match path.segments.last() {
        Some(seg) => seg,
        None => {
            return Err(syn::Error::new_spanned(
                &path.segments,
                "No segments in type!",
            ));
        }
    };
    let args = match &seg.arguments {
        syn::PathArguments::AngleBracketed(bracketed) => &bracketed.args,
        _ => {
            return Err(syn::Error::new_spanned(&seg.arguments, "No angle bracket!"));
            //return Ok(None);
        }
    };
    let arg = if args.len() == 1 {
        match &args[0] {
            syn::GenericArgument::Type(arg) => arg,
            _ => {
                //return Ok(None);
                return Err(syn::Error::new_spanned(
                    &seg.ident,
                    "No inner generic in type!",
                ));
            }
        }
    } else {
        return Err(syn::Error::new_spanned(
            &seg.ident,
            "Length of inner type args is not 1",
        ));
    };
    Ok(arg)
}

fn ungroup(mut ty: &syn::Type) -> &syn::Type {
    while let syn::Type::Group(group) = ty {
        ty = &group.elem;
    }
    ty
}

#[allow(dead_code)]
fn extract_type_path(ty: &syn::Type) -> Option<&syn::Path> {
    match *ty {
        syn::Type::Path(ref typepath) if typepath.qself.is_none() => Some(&typepath.path),
        _ => None,
    }
}
