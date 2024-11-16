//! # epics_gen_macros
//!
//! epics_gen_macros is an internal macro crate that contains macro definitions for the epics_gen library
//!

use syn::DeriveInput;
mod as_record;
mod xlsx;

//TODO: Implement multiple occurences of same attr error!

/// Convenience macro that implements FromXlsxString for marked type. It is used to automatically define functions needed
/// to convert calamine::Data::String to target type.
#[proc_macro_derive(FromXlsxString)]
pub fn derive_from_xstring(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast = syn::parse(input).unwrap();
    // Build the trait implementation
    xlsx::impl_derive_from_xstring(&ast).into()
}

/// Convenience macro that implements FromXlsxFloat for marked type. It is used to automatically define functions needed
/// to convert calamine::Data::Float to target type.
#[proc_macro_derive(FromXlsxFloat)]
pub fn derive_from_xfloat(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse(input).unwrap();
    xlsx::impl_derive_from_xfloat(&ast)
        .unwrap_or_else(|err| err.into_compile_error())
        .into()
}

/// Convenience macro that implements FromXlsxRow for marked type. It is used to automatically
/// convert XlsxRow (XlsxRow = Vec<calamine::Data>) to target type (usually a structure).
#[proc_macro_derive(FromXlsxRow)]
pub fn derive_from_xlsx_row(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse(input).unwrap();
    xlsx::impl_derive_xlsx_row(&ast)
        .unwrap_or_else(|err| err.into_compile_error())
        .into()
}

/// Implements `as_record` method for struct. (along with helper methods for each member)
///
/// Returns struct in form of EPICS records. Usually the `as_record` is implemented by `AsRecord`
/// derive proc_macro, but if some additional bussiness logic needs to be implemented, the
/// `as_record` for type `EvrOutput` can be implemented manually.
#[proc_macro_derive(AsRecord, attributes(record))]
pub fn derive_as_record(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse_macro_input!(input as DeriveInput);
    //let ast = syn::parse(input).unwrap();
    as_record::impl_derive_as_record(&ast)
        .unwrap_or_else(|err| err.into_compile_error())
        .into()
}
