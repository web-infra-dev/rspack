use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{Fields, ItemEnum, Variant};

pub fn expand(tokens: TokenStream) -> TokenStream {
  let mut item = match syn::parse2::<ItemEnum>(tokens) {
    Ok(v) => v,
    Err(err) => return err.into_compile_error(),
  };
  let variants = item
    .variants
    .iter_mut()
    .map(|p| {
      let v = p.clone();
      p.attrs.clear();
      v
    })
    .collect::<Vec<Variant>>();

  let original_name = &item.ident;
  let name = get_intermediate_ident(&original_name.to_string());
  let mut field_idents = vec![];
  let fields = variants
    .iter()
    .map(|f| {
      let attrs = &f.attrs;
      let ident = &f.ident;
      field_idents.push(ident);
      match &f.fields {
        Fields::Unnamed(field) => {
          let field = match field.unnamed.iter().next() {
            Some(f) => f,
            None => {
              return syn::Error::new(ident.span(), "Should have one field").into_compile_error()
            }
          };
          let ty = &field.ty;
          if let Some(field_ident) = &field.ident {
            quote! {
              #(#attrs)*
              pub #ident: Option<#field_ident>
            }
          } else {
            quote! {
              #(#attrs)*
              pub #ident: Option<#ty>
            }
          }
        }
        _ => syn::Error::new(
          ident.span(),
          "Unsupported fields type. Only fields of type `Fields::Unnamed` are supported.",
        )
        .into_compile_error(),
      }
    })
    .collect::<Vec<TokenStream>>();

  let union_type_name = Ident::new(&format!("{original_name}Type"), Span::call_site());

  let as_str_impl = {
    let arms = field_idents
      .iter()
      .map(|field| {
        let field_name = field.to_string();
        quote! {
          #union_type_name::#field => #field_name
        }
      })
      .collect::<Vec<TokenStream>>();

    quote! {
      impl #union_type_name {
        fn as_str(&self) -> &'static str {
           match self {
            #(#arms),*
          }
        }
      }
    }
  };

  let get_enum_impl = {
    field_idents
      .iter()
      .map(|field| {
        let s = field.to_string();
        quote! {
          if r#type == #s {
            let p = value.#field.take().unwrap_or_else(|| {
              panic!("Expected value of {} to exist", #s)
            });
            return #original_name::#field(p);
          }
        }
      })
      .collect::<Vec<TokenStream>>()
  };

  quote! {
    #[allow(non_camel_case_types)]
    #item

    #[napi(object, js_name = #original_name, object_to_js = false)]
    #[allow(non_snake_case)]
    pub struct #name {
      pub r#type: #union_type_name,
      #(#fields),*
    }

    #[napi(string_enum)]
    #[derive(Debug)]
    #[allow(non_camel_case_types)]
    pub enum #union_type_name {
      #(#field_idents),*
    }

    #as_str_impl

    impl From<#name> for #original_name {
      fn from(mut value: #name) -> Self {
        let r#type = value.r#type.as_str();
        #(
          #get_enum_impl
        )*
        panic!("Unknown type: {}", r#type);
      }
    }

    impl ::napi::bindgen_prelude::FromNapiValue for #original_name {
      unsafe fn from_napi_value(
        env: ::napi::sys::napi_env,
        napi_val: ::napi::sys::napi_value,
      ) -> ::napi::Result<Self> {
        let item = <#name as ::napi::bindgen_prelude::FromNapiValue>::from_napi_value(env, napi_val)?;
        Ok(item.into())
      }
    }
  }
}

fn get_intermediate_ident(name: &str) -> Ident {
  let new_name = format!("__rspack_napi__{name}");
  Ident::new(&new_name, Span::call_site())
}
