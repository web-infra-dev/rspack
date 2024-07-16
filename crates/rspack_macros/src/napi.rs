use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{parse_macro_input, ItemStruct, Meta, Visibility};

fn rm_raw_prefix(s: String) -> String {
  if let Some(stripped) = s.strip_prefix("r#") {
    stripped.to_string()
  } else {
    s
  }
}

pub fn getters(
  _args: proc_macro::TokenStream,
  tokens: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
  let mut input = parse_macro_input!(tokens as ItemStruct);

  let struct_name = &input.ident;
  let mut getters = Vec::with_capacity(input.fields.len());

  if let syn::Fields::Named(fields) = &mut input.fields {
    for named in &mut fields.named {
      if let Visibility::Inherited = named.vis {
        let field_name = named
          .ident
          .as_ref()
          .expect("Field name is missing. This should never happen in a named field structure.");

        let skip_getter_index = named.attrs.iter().position(|attr| {
          if let Meta::Path(path) = &attr.meta {
            if path.is_ident("skip_getter") {
              return true;
            }
          }
          false
        });
        if let Some(skip_getter_index) = skip_getter_index {
          named.attrs.remove(skip_getter_index);
          continue;
        }

        let method_name = Ident::new(
          &format!("get_{}", rm_raw_prefix(field_name.to_string())),
          Span::call_site(),
        );
        let ty = &named.ty;

        let mut getter = quote! {
            #[allow(clippy::clone_on_copy)]
            #[napi(getter)]
            pub fn #method_name(&self) -> #ty {
                self.#field_name.clone()
            }
        };
        if let syn::Type::Path(type_path) = &ty {
          let segment = &type_path
            .path
            .segments
            .first()
            .expect("Type path segment is missing. Ensure the field type is properly defined.");
          if segment.ident == "Option" {
            if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
              let inner_ty = args
                .args
                .first()
                .expect("Generic argument for Option type is missing.");
              getter = quote! {
                #[allow(clippy::clone_on_copy)]
                #[napi(getter)]
                pub fn #method_name(&self) -> rspack_napi::napi::Either<#inner_ty, ()> {
                  match &self.#field_name {
                      Some(f) => Either::A(f.clone()),
                      None => Either::B(()),
                  }
                }
              }
            }
          }
        };
        getters.push(getter);
      }
    }
  }

  quote! {
    #input

    #[napi]
    impl #struct_name {
        #(#getters)*
    }
  }
  .into()
}
