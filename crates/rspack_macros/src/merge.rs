use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned, ToTokens};
use syn::{
  parse_quote, spanned::Spanned, Attribute, Data, DeriveInput, Error, Fields, Generics, Ident,
  Index, Meta, Result,
};

pub fn expand_merge_from_derive(input: DeriveInput) -> Result<TokenStream> {
  let name = input.ident;
  let generics = add_trait_bounds(input.generics);
  let chose_base = input.attrs.iter().try_find(|a| -> Result<bool> {
    match &a.meta {
      Meta::List(list) => {
        if !list.path.is_ident("merge_from") {
          return Ok(false);
        }
        let mut chose_base = false;
        list.parse_nested_meta(|meta| {
          chose_base = meta.path.is_ident("enum_base");
          Ok(())
        })?;
        Ok(chose_base)
      }
      _ => Ok(false),
    }
  })?;
  let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
  let body = body(input.data, chose_base)?;
  Ok(quote! {
    impl #impl_generics rspack_util::MergeFrom for #name #ty_generics #where_clause {
      fn merge_from(#[allow(unused_mut)] mut self, other: &Self) -> Self {
        #body
      }
    }
  })
}

fn add_trait_bounds(mut generics: Generics) -> Generics {
  for param in &mut generics.params {
    if let syn::GenericParam::Type(ref mut type_param) = *param {
      type_param.bounds.push(parse_quote!(rspack_util::MergeFrom));
    }
  }
  generics
}

fn body(data: Data, chose_base: Option<&Attribute>) -> Result<TokenStream> {
  match data {
    Data::Struct(ref data) => match data.fields {
      Fields::Named(ref fields) => {
        let recurse = fields.named.iter().map(|f| {
          let name = &f.ident;
          quote_spanned! {f.span()=>
            self.#name = rspack_util::MergeFrom::merge_from(self.#name, &other.#name);
          }
        });
        Ok(quote! {
          #(#recurse)*
          self
        })
      }
      Fields::Unnamed(ref fields) => {
        let recurse = fields.unnamed.iter().enumerate().map(|(i, f)| {
          let index = Index::from(i);
          quote_spanned! {f.span()=>
            self.#index = rspack_util::MergeFrom::merge_from(self.#index, &other.#index);
          }
        });
        Ok(quote! {
          #(#recurse)*
          self
        })
      }
      Fields::Unit => Ok(quote! {
        self
      }),
    },
    Data::Enum(data) => {
      let recurse = data.variants.iter().map(|v| {
        let variant_name = &v.ident;
        match &v.fields {
          Fields::Named(f) => {
            let (fields, merged_fields): (Vec<(TokenStream, TokenStream)>, Vec<TokenStream>) = f
              .named
              .iter()
              .enumerate()
              .map(|(i, f)| {
                let field_name = &f.ident;
                let a_name = Ident::new(&format!("a_{i}"), field_name.span());
                let b_name = Ident::new(&format!("b_{i}"), field_name.span());
                (
                  (
                    quote_spanned! {field_name.span()=>
                      #field_name: #a_name
                    },
                    quote_spanned! {field_name.span()=>
                      #field_name: #b_name
                    },
                  ),
                  quote_spanned! {f.span()=>
                    #field_name: rspack_util::MergeFrom::merge_from(#a_name, #b_name)
                  },
                )
              })
              .unzip();
            let (a_fields, b_fields): (Vec<TokenStream>, Vec<TokenStream>) =
              fields.into_iter().unzip();
            quote_spanned! {v.span()=>
              (Self::#variant_name { #(#a_fields),* }, Self::#variant_name { #(#b_fields),* }) => {
                Self::#variant_name {
                  #(#merged_fields),*
                }
              }
            }
          }
          Fields::Unnamed(f) => {
            let (fields, merged_fields): (Vec<(TokenStream, TokenStream)>, Vec<TokenStream>) = f
              .unnamed
              .iter()
              .enumerate()
              .map(|(i, f)| {
                let a_name = Ident::new(&format!("a_{i}"), Span::call_site());
                let b_name = Ident::new(&format!("b_{i}"), Span::call_site());
                (
                  (a_name.to_token_stream(), b_name.to_token_stream()),
                  quote_spanned! {f.span()=>
                    rspack_util::MergeFrom::merge_from(#a_name, #b_name)
                  },
                )
              })
              .unzip();
            let (a_fields, b_fields): (Vec<TokenStream>, Vec<TokenStream>) =
              fields.into_iter().unzip();
            quote_spanned! {v.span()=>
              (Self::#variant_name(#(#a_fields),*), Self::#variant_name(#(#b_fields),*)) => {
                Self::#variant_name(#(#merged_fields),*)
              }
            }
          }
          Fields::Unit => {
            quote_spanned! {v.span()=>
              (Self::#variant_name, Self::#variant_name) => {
                Self::#variant_name
              }
            }
          }
        }
      });
      let fallback = if chose_base.is_some() {
        quote! {
          (a @ _, _) => a,
        }
      } else {
        quote! {
          (_, _) => other.clone(),
        }
      };
      Ok(quote! {
        match (self, other) {
          #(#recurse)*
          #fallback
        }
      })
    }
    Data::Union(data) => Err(Error::new_spanned(
      data.union_token,
      "MergeFrom is not implemented for unions",
    )),
  }
}
