use proc_macro::TokenStream;
use proc_macro2::Span;
use syn::{Fields, Ident, ItemStruct};
use quote::quote;

#[proc_macro_attribute]
pub fn ast(_args: TokenStream, input: TokenStream) -> TokenStream{
    let item = syn::parse::<ItemStruct>(input.clone());
    if item.is_err() {
        return quote! { compile_error!("#[ast] can only be applied to structs"); }.into();
    }
    let mut s=item.unwrap();

    let fields_named = match s.fields{
        Fields::Named(ref mut fields_named) => fields_named,
        _=>return quote! { compile_error!("#[ast] only supports structs with named fields"); }.into(),
    };

    let class_name = &s.ident;
    let new_field: syn::Field = syn::parse_quote! { name: String };
    fields_named.named.push(new_field);
    let _name_field = Ident::new("name", Span::call_site());

    let expend=quote! {
        #s

        impl crate::parser::ast::AstNode for #class_name{

        }
    };
    expend.into()
}