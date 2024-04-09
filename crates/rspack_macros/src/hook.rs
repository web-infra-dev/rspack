use proc_macro2::TokenStream;
use quote::quote;
use syn::{
  parse::{Parse, ParseStream},
  punctuated::Punctuated,
  token::Comma,
  Error, Ident, LitStr, PatType, Result, Token, TypePath,
};

pub struct DefineHookInput {
  trait_name: Ident,
  args: Punctuated<PatType, Comma>,
  exec_kind: ExecKind,
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
      "AsyncSeriesBail" => {
        <Token![->]>::parse(input)?;
        let ret = TypePath::parse(input)?;
        ExecKind::AsyncSeriesBail { ret }
      }
      "AsyncSeries" => ExecKind::AsyncSeries,
      "AsyncParallel" => ExecKind::AsyncParallel,
      "SyncSeries" => ExecKind::SyncSeries,
      _ => {
        return Err(Error::new_spanned(
          kind_ident,
          "unsupported hook execution kind",
        ))
      }
    };
    Ok(Self {
      trait_name,
      args,
      exec_kind,
    })
  }
}

impl DefineHookInput {
  pub fn expand(self) -> Result<TokenStream> {
    let DefineHookInput {
      trait_name,
      args,
      exec_kind,
    } = self;
    let ret = exec_kind.return_type();
    let is_async = exec_kind.is_async();
    let attr = if is_async {
      Some(quote! { #[::rspack_hook::__macro_helper::async_trait] })
    } else {
      None
    };
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
    let call_body = exec_kind.body(arg_names);
    let call_fn = quote! {
      fn call(&self, #args) -> #ret {
        #call_body
      }
    };
    let call_fn = if is_async {
      quote! { async #call_fn }
    } else {
      call_fn
    };
    let hook_name = Ident::new(&format!("{trait_name}Hook"), trait_name.span());
    let hook_name_lit_str = LitStr::new(&hook_name.to_string(), trait_name.span());
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
        interceptors: Vec<Box<dyn rspack_hook::Interceptor<Self> + Send + Sync>>,
      }

      impl rspack_hook::Hook for #hook_name {
        type Tap = Box<dyn #trait_name + Send + Sync>;

        fn used_stages(&self) -> rspack_hook::__macro_helper::FxHashSet<i32> {
          rspack_hook::__macro_helper::FxHashSet::from_iter(self.taps.iter().map(|h| h.stage()))
        }

        fn intercept(&mut self, interceptor: impl rspack_hook::Interceptor<Self> + Send + Sync + 'static) {
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
  AsyncSeries,
  AsyncSeriesBail { ret: TypePath },
  AsyncParallel,
  SyncSeries,
}

impl ExecKind {
  pub fn is_async(&self) -> bool {
    match self {
      Self::AsyncSeries | Self::AsyncSeriesBail { .. } | Self::AsyncParallel => true,
      Self::SyncSeries => false,
    }
  }

  pub fn return_type(&self) -> TokenStream {
    match self {
      Self::AsyncSeriesBail { ret } => {
        quote! { rspack_hook::__macro_helper::Result<std::option::Option<#ret>> }
      }
      _ => quote! { rspack_hook::__macro_helper::Result<()> },
    }
  }

  pub fn body(&self, args: Punctuated<&Ident, Comma>) -> TokenStream {
    match self {
      Self::AsyncSeries => {
        quote! {
          let mut additional_taps = std::vec::Vec::new();
          for interceptor in self.interceptors.iter() {
            additional_taps.extend(interceptor.call(self).await?);
          }
          let mut all_taps = std::vec::Vec::new();
          all_taps.extend(&additional_taps);
          all_taps.extend(&self.taps);
          all_taps.sort_by_key(|hook| hook.stage());
          for tap in all_taps {
            tap.run(#args).await?;
          }
          Ok(())
        }
      }
      Self::AsyncSeriesBail { .. } => {
        quote! {
          let mut additional_taps = std::vec::Vec::new();
          for interceptor in self.interceptors.iter() {
            additional_taps.extend(interceptor.call(self).await?);
          }
          let mut all_taps = std::vec::Vec::new();
          all_taps.extend(&additional_taps);
          all_taps.extend(&self.taps);
          all_taps.sort_by_key(|hook| hook.stage());
          for hook in all_taps {
            if let Some(res) = hook.run(#args).await? {
              return Ok(Some(res));
            }
          }
          Ok(None)
        }
      }
      Self::AsyncParallel => {
        quote! {
          let mut additional_taps = std::vec::Vec::new();
          for interceptor in self.interceptors.iter() {
            additional_taps.extend(interceptor.call(self).await?);
          }
          let mut all_taps = std::vec::Vec::new();
          all_taps.extend(&additional_taps);
          all_taps.extend(&self.taps);
          all_taps.sort_by_key(|hook| hook.stage());
          let futs: std::vec::Vec<_> = all_taps.iter().map(|t| t.run(#args)).collect();
          futures_concurrency::vec::TryJoin(futs).await?;
          Ok(())
        }
      }
      Self::SyncSeries => {
        quote! {
          let mut additional_taps = std::vec::Vec::new();
          for interceptor in self.interceptors.iter() {
            additional_taps.extend(interceptor.call_blocking(self)?);
          }
          let mut all_taps = std::vec::Vec::new();
          all_taps.extend(&additional_taps);
          all_taps.extend(&self.taps);
          all_taps.sort_by_key(|hook| hook.stage());
          for tap in all_taps {
            tap.run(#args)?;
          }
          Ok(())
        }
      }
    }
  }
}
