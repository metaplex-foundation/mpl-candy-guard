use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(CandyGuard)]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;

    let fields = if let syn::Data::Struct(syn::DataStruct {
        fields: syn::Fields::Named(syn::FieldsNamed { ref named, .. }),
        ..
    }) = ast.data
    {
        named
    } else {
        unimplemented!();
    };

    let unwrap_option_t = |ty: &syn::Type| -> syn::Type {
        if let syn::Type::Path(ref p) = ty {
            if let syn::PathArguments::AngleBracketed(ref inner_ty) = p.path.segments[0].arguments {
                if inner_ty.args.len() != 1 {
                    panic!("Option type was not Option<T>");
                } else if let syn::GenericArgument::Type(ref ty) = inner_ty.args.first().unwrap() {
                    ty.clone()
                } else {
                    panic!("Option type was not Option<T>");
                }
            } else {
                panic!("Option type was not Option<T>");
            }
        } else {
            panic!("Type was not Option<T>");
        }
    };

    let from_data = fields.iter().map(|f| {
        let name = &f.ident;
        let ty = unwrap_option_t(&f.ty);
        quote! {
            current += #ty::size();
            let #name = if #ty::is_enabled(features) {
                #ty::load(data, current)?
            } else {
                None
            };
        }
    });

    let to_data = fields.iter().map(|f| {
        let name = &f.ident;
        let ty = unwrap_option_t(&f.ty);
        quote! {
            offset += #ty::size();
            if let Some(#name) = &self.#name {
                if offset <= data.len() {
                    #name.save(data, offset - #ty::size())?;
                    features = #ty::enable(features);
                }
            }
        }
    });

    let struct_fields = fields.iter().map(|f| {
        let name = &f.ident;
        quote! { #name }
    });

    let enabled = fields.iter().map(|f| {
        let name = &f.ident;
        quote! {
            if let Some(#name) = &self.#name {
                conditions.push(#name);
            }
        }
    });

    let struct_length = fields.iter().map(|f| {
        let ty = unwrap_option_t(&f.ty);
        quote! { length += #ty::size(); }
    });

    let expanded = quote! {
        impl #name {
            pub fn from_data(features: u64, data: &mut std::cell::RefMut<&mut [u8]>) -> anchor_lang::Result<Self> {
                let mut current = DATA_OFFSET;

                #(#from_data)*

                Ok(Self {
                    #(#struct_fields,)*
                })
            }

            pub fn to_data(&self, data: &mut [u8]) -> anchor_lang::Result<u64> {
                let mut features = 0;
                let mut offset = 0;

                #(#to_data)*

                Ok(features)
            }

            pub fn enabled_conditions(&self) -> Vec<&dyn Condition> {
                // list of condition trait objects
                let mut conditions: Vec<&dyn Condition> = vec![];
                #(#enabled)*

                conditions
            }

            pub fn data_length() -> usize {
                let mut length = DATA_OFFSET;
                #(#struct_length)*
                length
            }
        }
    };

    TokenStream::from(expanded)
}
