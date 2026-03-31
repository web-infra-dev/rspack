use quote::quote;
use syn::{ImplItem, ItemImpl, Result, parse_macro_input, parse_quote};

pub fn expand(
  args: proc_macro::TokenStream,
  tokens: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
  if !proc_macro2::TokenStream::from(args).is_empty() {
    return syn::Error::new(
      proc_macro2::Span::call_site(),
      "attribute does not accept arguments",
    )
    .to_compile_error()
    .into();
  }

  let mut input = parse_macro_input!(tokens as ItemImpl);
  match expand_impl(&mut input) {
    Ok(()) => quote!(#input).into(),
    Err(err) => err.to_compile_error().into(),
  }
}

fn expand_impl(input: &mut ItemImpl) -> Result<()> {
  let Some((_, trait_path, _)) = &input.trait_ else {
    return Err(syn::Error::new_spanned(
      &input.self_ty,
      "expected a trait impl for JavascriptParserPlugin",
    ));
  };

  if trait_path
    .segments
    .last()
    .is_none_or(|segment| segment.ident != "JavascriptParserPlugin")
  {
    return Err(syn::Error::new_spanned(
      trait_path,
      "attribute only supports impl JavascriptParserPlugin for ...",
    ));
  }

  let mut hook_variants = Vec::new();
  for item in &input.items {
    let ImplItem::Fn(func) = item else {
      continue;
    };

    let method_name = func.sig.ident.to_string();
    let normalized_name = method_name.strip_prefix("r#").unwrap_or(&method_name);
    if normalized_name == "implemented_hooks" || normalized_name == "hooks" {
      return Err(syn::Error::new_spanned(
        &func.sig.ident,
        "remove manual hook metadata; this attribute generates it automatically",
      ));
    }

    hook_variants.push(hook_variant_ident(&func.sig.ident)?);
  }

  let body = if let Some(first) = hook_variants.first() {
    let rest = &hook_variants[1..];
    quote! {
      ::rspack_plugin_javascript::JavascriptParserPluginHooks::empty()
        .with(::rspack_plugin_javascript::JavascriptParserPluginHook::#first)
        #(.with(::rspack_plugin_javascript::JavascriptParserPluginHook::#rest))*
    }
  } else {
    quote! {
      ::rspack_plugin_javascript::JavascriptParserPluginHooks::empty()
    }
  };

  input.items.insert(
    0,
    parse_quote! {
      fn implemented_hooks(&self) -> ::rspack_plugin_javascript::JavascriptParserPluginHooks {
        #body
      }
    },
  );

  Ok(())
}

fn hook_variant_ident(method_ident: &syn::Ident) -> Result<syn::Ident> {
  let method_name = method_ident.to_string();
  let normalized_name = method_name.strip_prefix("r#").unwrap_or(&method_name);

  let mut variant_name = String::with_capacity(normalized_name.len());
  let mut uppercase_next = true;

  for ch in normalized_name.chars() {
    if ch == '_' {
      uppercase_next = true;
      continue;
    }

    if uppercase_next {
      variant_name.extend(ch.to_uppercase());
      uppercase_next = false;
    } else {
      variant_name.push(ch);
    }
  }

  if variant_name.is_empty() {
    return Err(syn::Error::new_spanned(
      method_ident,
      "failed to derive parser hook variant from method name",
    ));
  }

  Ok(syn::Ident::new(&variant_name, method_ident.span()))
}
