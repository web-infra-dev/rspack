use proc_macro2::{Ident, Literal, TokenStream};
use quote::{format_ident, quote};
use syn::{
  Block, FnArg, Signature, Token, Visibility,
  fold::{Fold, fold_fn_arg, fold_signature},
  parse::{Parse, ParseStream, Result},
  punctuated::Punctuated,
};

pub struct ArgLength {
  pub length: Literal,
}

impl Parse for ArgLength {
  fn parse(input: ParseStream) -> Result<Self> {
    let vars = Punctuated::<Literal, Token![,]>::parse_terminated(input)?;
    Ok(ArgLength {
      length: vars
        .first()
        .cloned()
        .unwrap_or_else(|| Literal::usize_unsuffixed(0)),
    })
  }
}

pub struct JsFunction {
  pub args: Vec<FnArg>,
  pub name: Option<Ident>,
  pub signature: Option<Signature>,
  pub signature_raw: Option<Signature>,
  pub block: Vec<Block>,
  pub visibility: Visibility,
}

impl JsFunction {
  pub fn new() -> Self {
    JsFunction {
      args: vec![],
      name: None,
      signature: None,
      signature_raw: None,
      visibility: Visibility::Inherited,
      block: vec![],
    }
  }
}

impl Fold for JsFunction {
  fn fold_fn_arg(&mut self, arg: FnArg) -> FnArg {
    self.args.push(arg.clone());
    fold_fn_arg(self, arg)
  }

  fn fold_signature(&mut self, signature: Signature) -> Signature {
    self.name = Some(format_ident!("{}", signature.ident));
    let mut new_signature = signature.clone();
    new_signature.ident = format_ident!("_generated_{}_generated_", signature.ident);
    self.signature = Some(new_signature);
    self.signature_raw = Some(signature.clone());
    fold_signature(self, signature)
  }

  fn fold_visibility(&mut self, v: Visibility) -> Visibility {
    self.visibility = v.clone();
    v
  }

  fn fold_block(&mut self, node: Block) -> Block {
    self.block.push(node.clone());
    node
  }
}
pub enum FunctionKind {
  Contextless,
  JsFunction,
}

pub fn get_execute_js_code(new_fn_name: Ident, function_kind: FunctionKind) -> TokenStream {
  let return_token_stream = match function_kind {
    FunctionKind::Contextless => {
      quote! {
        Ok(Some(v)) => unsafe {
          napi::bindgen_prelude::ToNapiValue::to_napi_value(raw_env, v).unwrap_or_else(|e| {
            napi::JsError::from(e).throw_into(raw_env);
            ptr::null_mut()
          })
        },
        Ok(None) => ptr::null_mut(),
      }
    }
    FunctionKind::JsFunction => {
      quote! {
        Ok(v) => unsafe {
          napi::bindgen_prelude::ToNapiValue::to_napi_value(raw_env, v).unwrap_or_else(|e| {
            napi::JsError::from(e).throw_into(raw_env);
            ptr::null_mut()
          })
        },
      }
    }
  };
  quote! {
    match panic::catch_unwind(AssertUnwindSafe(move || #new_fn_name(ctx))).map_err(|e| {
      let message = {
        if let Some(string) = e.downcast_ref::<String>() {
          string.clone()
        } else if let Some(string) = e.downcast_ref::<&str>() {
          string.to_string()
        } else {
          format!("panic from Rust code: {:?}", e)
        }
      };
      Error::from_reason(message)
    }).and_then(|v| v) {
      #return_token_stream
      Err(e) => {
        unsafe { napi::JsError::from(e).throw_into(raw_env) };
        ptr::null_mut()
      }
    }
  }
}
