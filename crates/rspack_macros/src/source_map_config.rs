use quote::quote;
use syn::{parse::Parser, parse_macro_input, ItemStruct};

pub fn impl_source_map_config(
  _args: proc_macro::TokenStream,
  tokens: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
  let mut input = parse_macro_input!(tokens as ItemStruct);
  let name = &input.ident;
  let generics = &input.generics;
  let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

  if let syn::Fields::Named(ref mut fields) = input.fields {
    fields.named.push(
      syn::Field::parse_named
        .parse2(quote! { pub source_map_kind: ::rspack_util::source_map::SourceMapKind })
        .expect("Failed to parse new field for source_map_kind"),
    );
  }

  quote! {
    #input

    impl #impl_generics ::rspack_util::source_map::ModuleSourceMapConfig for #name #ty_generics #where_clause {
      fn get_source_map_kind(&self) -> &::rspack_util::source_map::SourceMapKind {
        &self.source_map_kind
      }

      fn set_source_map_kind(&mut self, source_map_kind: ::rspack_util::source_map::SourceMapKind) {
        self.source_map_kind = source_map_kind;
      }
    }
  }
  .into()
}
