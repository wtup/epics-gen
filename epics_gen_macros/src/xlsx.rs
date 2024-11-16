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
        unimplemented!();
    };
    let mut field_convert = Vec::new();

    for (i, field) in fields.iter().enumerate() {
        let id = &field.ident;

        if let syn::Type::Path(syn::TypePath {
            path: syn::Path { segments, .. },
            ..
        }) = &field.ty
        {
            // The value is wrapped in option!
            let field_output = if !segments.is_empty() && segments.last().unwrap().ident == "Option"
            {
                let last_segment = segments.last().unwrap();
                let inner_type =
                    if let syn::PathArguments::AngleBracketed(generics) = &last_segment.arguments {
                        if generics.args.len() != 1 {
                            //TODO: Error handling
                            panic!("No generic type available!");
                        }
                        let syn::GenericArgument::Type(inner_type) = &generics.args[0] else {
                            //TODO: Error handling
                            panic!("Wrong inner type!");
                        };
                        inner_type
                    } else {
                        //TODO: Error handling
                        panic!("Illegal bracket used!");
                    };
                quote! {
                    #id: {
                        let val = row.pop().unwrap();
                        let res = match #inner_type::try_from(val.clone()) {
                            Err(epics_gen::ParseErrorKind::ValueMissing) => None,
                            v => Some(v.map_err(|kind| epics_gen::ParseError::new_in_table(kind,epics_gen::XlsxCell::new((row_num as u32, #i as u32), val), table_name.to_owned()))?),
                        };
                        res
                    }
                }
            // The value is not wrapped in option
            } else {
                quote! {
                    #id: {
                        let val = row.pop().unwrap();
                        val.clone().try_into().map_err(|kind| epics_gen::ParseError::new_in_table(kind,epics_gen::XlsxCell::new((row_num as u32, #i as u32), val), table_name.to_owned()))?
                    }
                }
            };
            field_convert.push(field_output);
        }
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
