use proc_macro2::TokenStream;
use quote::quote;
use syn::{
  Error, Ident, LitStr, PatType, Result, Token, TypePath,
  parse::{Parse, ParseStream},
  punctuated::Punctuated,
  token::Comma,
};

pub struct DefineHookInput {
  trait_name: Ident,
  args: Punctuated<PatType, Comma>,
  exec_kind: ExecKind,
  tracing: Option<syn::LitBool>,
}

impl Parse for DefineHookInput {
  fn parse(input: ParseStream) -> Result<Self> {
    let trait_name = Ident::parse(input)?;
    <Token![:]>::parse(input)?;
    let kind_ident = Ident::parse(input)?;
    let kind = kind_ident.to_string();
    let content;
    syn::parenthesized!(content in input);
    let args = content.parse_terminated(PatType::parse, Token![,])?;
    let exec_kind = match kind.as_str() {
      "SeriesBail" => ExecKind::SeriesBail {
        ret: ExecKind::parse_ret(input)?,
      },
      "SeriesWaterfall" => {
        let ret = match ExecKind::parse_ret(input)? {
          Some(t) => t,
          None => {
            return Err(Error::new(
              input.span(),
              "Waterfall hooks must explicitly define a return type",
            ));
          }
        };
        ExecKind::SeriesWaterfall { ret }
      }
      "Series" => ExecKind::Series,
      "Parallel" => ExecKind::Parallel,
      _ => {
        return Err(Error::new_spanned(
          kind_ident,
          "unsupported hook execution kind",
        ));
      }
    };

    let mut tracing = None;
    while input.peek(Token![,]) {
      input.parse::<Token![,]>()?;
      let ident = input.parse::<syn::Ident>()?;
      input.parse::<Token![=]>()?;

      match ident.to_string().as_str() {
        "tracing" => {
          tracing = Some(input.parse()?);
        }
        _ => return Err(input.error("expected \"tracing\" or end of attribute")),
      }
    }

    Ok(Self {
      trait_name,
      args,
      exec_kind,
      tracing,
    })
  }
}

impl DefineHookInput {
  pub fn expand(self) -> Result<TokenStream> {
    let DefineHookInput {
      trait_name,
      args,
      exec_kind,
      tracing,
    } = self;
    let ret = exec_kind.return_type();
    let attr = quote! { #[::rspack_hook::__macro_helper::async_trait] };
    let run_sig = quote! { fn run(&self, #args) -> #ret; };
    let run_sig = quote! { async #run_sig };
    let arg_names = args
      .iter()
      .map(|arg| match &*arg.pat {
        syn::Pat::Ident(pat) => Ok(&pat.ident),
        _ => Err(Error::new_spanned(arg, "unexpected arg")),
      })
      .collect::<Result<Punctuated<&Ident, Comma>>>()?;
    let hook_name = Ident::new(&format!("{trait_name}Hook"), trait_name.span());
    let hook_name_lit_str = LitStr::new(&hook_name.to_string(), trait_name.span());
    let call_body = exec_kind.body(arg_names);
    let call_body = if tracing.is_none_or(|bool_lit| bool_lit.value) {
      let tracing_span_name = LitStr::new(&format!("hook:{trait_name}"), trait_name.span());
      quote! {
        ::rspack_hook::__macro_helper::tracing::Instrument::instrument(
          async { #call_body },
          ::rspack_hook::__macro_helper::tracing::info_span!(#tracing_span_name),
        ).await
      }
    } else {
      call_body
    };
    let call_fn = quote! {
      async fn call(&self, #args) -> #ret {
        #call_body
      }
    };
    Ok(quote! {
      #attr
      pub trait #trait_name {
        #run_sig
        fn stage(&self) -> i32 {
          0
        }
      }

      pub struct #hook_name {
        taps: Vec<Box<dyn #trait_name + Send + Sync>>,
        interceptors: Vec<Box<dyn ::rspack_hook::Interceptor<Self> + Send + Sync>>,
      }

      impl ::rspack_hook::Hook for #hook_name {
        type Tap = Box<dyn #trait_name + Send + Sync>;

        fn used_stages(&self) -> ::rspack_hook::__macro_helper::FxHashSet<i32> {
          ::rspack_hook::__macro_helper::FxHashSet::from_iter(self.taps.iter().map(|h| h.stage()))
        }

        fn intercept(&mut self, interceptor: impl ::rspack_hook::Interceptor<Self> + Send + Sync + 'static) {
          self.interceptors.push(Box::new(interceptor));
        }
      }

      impl std::fmt::Debug for #hook_name {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
          write!(f, #hook_name_lit_str)
        }
      }

      impl Default for #hook_name {
        fn default() -> Self {
          Self {
            taps: Default::default(),
            interceptors: Default::default(),
          }
        }
      }

      impl #hook_name {
        pub #call_fn

        pub fn tap(&mut self, tap: impl #trait_name + Send + Sync + 'static) {
          self.taps.push(Box::new(tap));
        }
      }
    })
  }
}

enum ExecKind {
  Series,
  SeriesBail { ret: Option<TypePath> },
  SeriesWaterfall { ret: TypePath },
  Parallel,
}

impl ExecKind {
  pub fn parse_ret(input: ParseStream) -> Result<Option<TypePath>> {
    Ok(if input.peek(Token![->]) {
      <Token![->]>::parse(input)?;
      let ret = TypePath::parse(input)?;
      Some(ret)
    } else {
      None
    })
  }

  pub fn return_type(&self) -> TokenStream {
    match self {
      Self::SeriesBail { ret } => {
        if let Some(ret) = ret {
          quote! { ::rspack_hook::__macro_helper::Result<std::option::Option<#ret>> }
        } else {
          quote! { ::rspack_hook::__macro_helper::Result<std::option::Option<()>> }
        }
      }
      Self::SeriesWaterfall { ret } => {
        quote! { ::rspack_hook::__macro_helper::Result<#ret> }
      }
      _ => quote! { ::rspack_hook::__macro_helper::Result<()> },
    }
  }

  fn additional_taps(&self) -> TokenStream {
    let call = quote! { additional_taps.extend(interceptor.call(self).await?); };
    quote! {
      let mut additional_taps = std::vec::Vec::new();
      for interceptor in self.interceptors.iter() {
        #call
      }
      let mut all_taps = std::vec::Vec::with_capacity(self.taps.len() + additional_taps.len());
      all_taps.extend(&self.taps);
      all_taps.extend(&additional_taps);
      all_taps.sort_by_key(|hook| hook.stage());
    }
  }

  pub fn body(&self, args: Punctuated<&Ident, Comma>) -> TokenStream {
    let additional_taps = self.additional_taps();
    match self {
      Self::Series => {
        quote! {
          #additional_taps
          for tap in all_taps {
            tap.run(#args).await?;
          }
          Ok(())
        }
      }
      Self::SeriesBail { .. } => {
        quote! {
          #additional_taps
          for tap in all_taps {
            if let Some(res) = tap.run(#args).await? {
              return Ok(Some(res));
            }
          }
          Ok(None)
        }
      }
      Self::SeriesWaterfall { .. } => {
        quote! {
          #additional_taps
          let mut data = #args;
          for tap in all_taps {
            data = tap.run(data).await?
          }
          Ok(data)
        }
      }
      Self::Parallel => {
        quote! {
          #additional_taps
          let futs: std::vec::Vec<_> = all_taps.iter().map(|t| t.run(#args)).collect();
          futures_concurrency::vec::TryJoin(futs).await?;
          Ok(())
        }
      }
    }
  }
}
