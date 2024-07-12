use proc_macro2::Span;
use quote::quote;
use syn::{
  parse::{Parse, ParseStream, Parser},
  Result, Token,
};

pub fn expand_struct(mut input: syn::ItemStruct) -> proc_macro::TokenStream {
  let ident = &input.ident;
  let inner_ident = plugin_inner_ident(ident);
  let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
  let inner_fields = input.fields.clone();
  let is_named_struct = matches!(&inner_fields, syn::Fields::Named(_));
  let is_unit_struct = matches!(&inner_fields, syn::Fields::Unit);
  if !is_named_struct && !is_unit_struct {
    return syn::Error::new_spanned(inner_fields, "expected named struct or unit struct")
      .to_compile_error()
      .into();
  }

  input.fields = syn::Fields::Named(
    syn::FieldsNamed::parse
      .parse2(quote! { { inner: ::std::sync::Arc<#inner_ident #ty_generics> } })
      .expect("Failed to parse"),
  );

  let new_inner_fn = if is_named_struct {
    let field_names: Vec<&syn::Ident> = inner_fields
      .iter()
      .map(|field| field.ident.as_ref().expect("expected named struct"))
      .collect();
    let field_tys: Vec<&syn::Type> = inner_fields.iter().map(|field| &field.ty).collect();
    quote! {
      #[allow(clippy::too_many_arguments)]
      fn new_inner(#(#field_names: #field_tys,)*) -> Self {
        Self {
          inner: ::std::sync::Arc::new(#inner_ident { #(#field_names,)* }),
        }
      }
    }
  } else {
    quote! {
      fn new_inner() -> Self {
        Self {
          inner: ::std::sync::Arc::new(#inner_ident),
        }
      }
    }
  };

  let attrs = &input.attrs;

  let inner_struct = if is_named_struct {
    quote! {
      pub struct #inner_ident #impl_generics #where_clause #inner_fields
    }
  } else {
    quote! {
      pub struct #inner_ident #impl_generics #where_clause;
    }
  };

  let expanded = quote! {
    #input

    impl #impl_generics #ident #ty_generics #where_clause {
      #new_inner_fn

      fn from_inner(inner: &::std::sync::Arc<#inner_ident #ty_generics>) -> Self {
        Self {
          inner: ::std::sync::Arc::clone(inner),
        }
      }

      fn inner(&self) -> &::std::sync::Arc<#inner_ident #ty_generics> {
        &self.inner
      }
    }

    impl #impl_generics ::std::ops::Deref for #ident #ty_generics #where_clause {
      type Target = #inner_ident #ty_generics;
      fn deref(&self) -> &Self::Target {
        &self.inner
      }
    }

    #[doc(hidden)]
    #(#attrs)*
    #inner_struct
  };
  expanded.into()
}

fn plugin_inner_ident(ident: &syn::Ident) -> syn::Ident {
  let inner_name = format!("{}Inner", ident);
  syn::Ident::new(&inner_name, ident.span())
}

pub struct HookArgs {
  trait_: syn::Path,
  name: syn::Ident,
  stage: Option<syn::Expr>,
  tracing: Option<syn::LitBool>,
  generics: syn::Generics,
}

impl Parse for HookArgs {
  fn parse(input: ParseStream) -> Result<Self> {
    let trait_ = input.parse::<syn::Path>()?;
    input.parse::<Token![for]>()?;
    let name = input.parse::<syn::Ident>()?;
    let generics = input.parse::<syn::Generics>()?;

    let mut stage = None;
    let mut tracing = None;

    while input.peek(Token![,]) {
      input.parse::<Token![,]>()?;
      let ident = input.parse::<syn::Ident>()?;
      input.parse::<Token![=]>()?;

      match ident.to_string().as_str() {
        "stage" => {
          stage = Some(input.parse()?);
        }
        "tracing" => {
          tracing = Some(input.parse()?);
        }
        _ => return Err(input.error("expected \"stage\" or \"tracing\" or end of attribute")),
      }
    }

    Ok(Self {
      trait_,
      name,
      stage,
      generics,
      tracing,
    })
  }
}

pub fn expand_fn(args: HookArgs, input: syn::ItemFn) -> proc_macro::TokenStream {
  let HookArgs {
    name,
    trait_,
    stage,
    generics,
    tracing,
  } = args;
  let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
  let syn::ItemFn {
    mut sig,
    block,
    vis,
    ..
  } = input;
  let real_sig = sig.clone();
  let mut rest_args = Vec::new();
  for arg in real_sig.inputs.iter().skip(1) {
    if let syn::FnArg::Typed(syn::PatType { pat, .. }) = arg {
      rest_args.push(pat)
    } else {
      return syn::Error::new_spanned(arg, "unsupported syntax")
        .to_compile_error()
        .into();
    }
  }
  let is_async = sig.asyncness.is_some();
  let fn_ident = sig.ident.clone();
  sig.ident = syn::Ident::new("run", fn_ident.span());

  let inner_ident = plugin_inner_ident(&name);

  let tracing_name = syn::LitStr::new(&format!("{}::{}", &name, &fn_ident), Span::call_site());
  let tracing_annotation = tracing
    .map(|bool_lit| bool_lit.value)
    .unwrap_or(true)
    .then(|| {
      quote! {
        #[tracing::instrument(name = #tracing_name, skip_all)]
      }
    });

  let stage_fn = stage.map(|stage| {
    quote! {
      fn stage(&self) -> i32 {
        #stage
      }
    }
  });

  let attr = if is_async {
    Some(quote! { #[::rspack_hook::__macro_helper::async_trait] })
  } else {
    None
  };

  let call_real_fn = if is_async {
    quote! { #name::#fn_ident(&#name::from_inner(&self.inner), #(#rest_args,)*).await }
  } else {
    quote! { #name::#fn_ident(&#name::from_inner(&self.inner), #(#rest_args,)*) }
  };

  let expanded = quote! {
    #[allow(non_camel_case_types)]
    #vis struct #fn_ident #impl_generics #where_clause {
      #vis inner: ::std::sync::Arc<#inner_ident #ty_generics>,
    }

    impl #impl_generics #fn_ident #ty_generics #where_clause {
      #vis fn new(plugin: &#name #ty_generics) -> Self {
        #fn_ident {
          inner: ::std::sync::Arc::clone(plugin.inner()),
        }
      }
    }

    impl #impl_generics #name #ty_generics #where_clause {
      #[allow(clippy::ptr_arg)]
      #real_sig #block
    }

    impl #impl_generics ::std::ops::Deref for #fn_ident #ty_generics #where_clause {
      type Target = #inner_ident #ty_generics;
      fn deref(&self) -> &Self::Target {
        &self.inner
      }
    }

    #attr
    impl #impl_generics #trait_ for #fn_ident #ty_generics #where_clause {
      #tracing_annotation
      #sig {
        #call_real_fn
      }

      #stage_fn
    }
  };
  expanded.into()
}
