//! Rspack's only change to napi-derive 3.5.10: instrument completed class
//! conversions, after napi-rs has placed the Rust value at its final address.
//! Keep the upstream parser/codegen and TypeScript generation unchanged.
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Expr, GenericArgument, PathArguments, visit_mut::VisitMut};

pub fn instrument(tokens: TokenStream) -> syn::Result<TokenStream> {
  // Other users of this workspace alias do not necessarily depend on rspack_napi.
  if std::env::var("CARGO_PKG_NAME").as_deref() != Ok("rspack_binding_api") {
    return Ok(tokens);
  }
  let mut file = syn::parse2::<syn::File>(tokens)?;
  Instrument.visit_file_mut(&mut file);
  Ok(quote!(#file))
}

struct Instrument;
impl VisitMut for Instrument {
  fn visit_expr_mut(&mut self, expr: &mut Expr) {
    syn::visit_mut::visit_expr_mut(self, expr);
    let replacement = match expr {
      Expr::Call(call) => {
        let Expr::Path(path) = &*call.func else {
          return;
        };
        let Some(last) = path.path.segments.last() else {
          return;
        };
        if last.ident != "new_instance" {
          return;
        }
        let PathArguments::AngleBracketed(generics) = &last.arguments else {
          return;
        };
        let args = &call.args;
        quote!(::rspack_napi::allocative::new_instance #generics (#args))
      }
      Expr::MethodCall(call)
        if (call.method == "construct" || call.method == "factory")
          && matches!(&*call.receiver, Expr::Path(path) if path.path.is_ident("cb")) =>
      {
        let mut generics = call
          .turbofish
          .clone()
          .unwrap_or_else(|| syn::parse_quote!(::<_>));
        generics
          .args
          .push(GenericArgument::Type(syn::parse_quote!(_)));
        let method = &call.method;
        let receiver = &call.receiver;
        let args = &call.args;
        quote!(::rspack_napi::allocative::#method #generics (&#receiver, env, #args))
      }
      _ => return,
    };
    let original = expr.clone();
    *expr = syn::parse_quote!({
      #[cfg(allocative)] { unsafe { #replacement } }
      #[cfg(not(allocative))] { #original }
    });
  }
}
