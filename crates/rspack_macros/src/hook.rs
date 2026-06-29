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
      "Sync" => ExecKind::Sync,
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
    let is_async = exec_kind.is_async();
    let ret = exec_kind.return_type();
    let attr = is_async.then(|| quote! { #[::rspack_hook::__macro_helper::async_trait] });
    let run_sig = quote! { fn run(&self, #args) -> #ret; };
    let run_sig = if is_async {
      quote! { async #run_sig }
    } else {
      run_sig
    };
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
      if is_async {
        quote! {
          ::rspack_hook::__macro_helper::tracing::Instrument::instrument(
            async { #call_body },
            ::rspack_hook::__macro_helper::tracing::info_span!(#tracing_span_name),
          ).await
        }
      } else {
        quote! {
          let tracing_span = ::rspack_hook::__macro_helper::tracing::info_span!(#tracing_span_name);
          let _tracing_span_guard = tracing_span.enter();
          #call_body
        }
      }
    } else {
      call_body
    };
    let call_fn = if is_async {
      quote! {
        async fn call(&self, #args) -> #ret {
          #call_body
        }
      }
    } else {
      quote! {
        fn call(&self, #args) -> #ret {
          #call_body
        }
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
        taps: ::rspack_hook::__macro_helper::HookTaps<Self>,
      }

      impl ::rspack_hook::Hook for #hook_name {
        type Tap = Box<dyn #trait_name + Send + Sync>;

        fn tap_stage(tap: &Self::Tap) -> i32 {
          tap.stage()
        }

        fn used_stages(&self) -> ::rspack_hook::__macro_helper::FxHashSet<i32> {
          self.taps.used_stages()
        }

        fn intercept(&mut self, interceptor: impl ::rspack_hook::Interceptor<Self> + Send + Sync + 'static) {
          self.taps.intercept(interceptor);
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
          }
        }
      }

      impl #hook_name {
        pub #call_fn

        pub fn tap(&mut self, tap: impl #trait_name + Send + Sync + 'static) {
          self.taps.tap(Box::new(tap));
        }

        pub fn is_empty(&self) -> bool {
          self.taps.is_empty()
        }
      }
    })
  }
}

enum ExecKind {
  Series,
  Sync,
  SeriesBail { ret: Option<TypePath> },
  SeriesWaterfall { ret: TypePath },
  Parallel,
}

impl ExecKind {
  fn is_async(&self) -> bool {
    !matches!(self, Self::Sync)
  }

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
    let call = if self.is_async() {
      quote! { self.taps.call_interceptors(self).await? }
    } else {
      quote! { self.taps.call_interceptors_blocking(self)? }
    };
    quote! {
      let additional_taps = #call;
      let all_taps = self.taps.sorted_taps(&additional_taps);
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
      Self::Sync => {
        quote! {
          #additional_taps
          for tap in all_taps {
            tap.run(#args)?;
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
