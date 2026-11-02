#[macro_use]
pub mod attrs;

use std::{
  collections::{HashMap, HashSet},
  str::Chars,
  sync::{LazyLock, Mutex, OnceLock, atomic::AtomicUsize},
};

use attrs::BindgenAttrs;
use convert_case::Case;
use napi_derive_backend::{
  BindgenResult, CallbackArg, Diagnostic, FnKind, FnSelf, Napi, NapiArray, NapiClass, NapiConst,
  NapiEnum, NapiEnumValue, NapiEnumVariant, NapiFn, NapiFnArg, NapiFnArgKind, NapiImpl, NapiItem,
  NapiObject, NapiStruct, NapiStructField, NapiStructKind, NapiStructuredEnum,
  NapiStructuredEnumVariant, NapiTransparent, NapiType, rm_raw_prefix, to_case,
};
use proc_macro2::{Ident, Span, TokenStream};
use quote::ToTokens;
use syn::{
  AngleBracketedGenericArguments, Attribute, ExprLit, GenericArgument, Meta, PatType, Path,
  PathArguments, PathSegment, Signature, Token, Type, Visibility,
  ext::IdentExt,
  parse::{Parse, ParseStream, Result as SynResult},
  spanned::Spanned,
};

use crate::parser::attrs::{check_recorded_struct_for_impl, record_struct};

/// Stores (is_sync_generator, is_async_generator) for each struct
static GENERATOR_STRUCT: OnceLock<Mutex<HashMap<String, (bool, bool)>>> = OnceLock::new();

static REGISTER_INDEX: AtomicUsize = AtomicUsize::new(0);

static KNOWN_JS_VALUE_TYPES_WITH_LIFETIME: LazyLock<HashSet<&str>> = LazyLock::new(|| {
  [
    "Array",
    "Function",
    "JsDate",
    "JsGlobal",
    "JsNumber",
    "JsString",
    "JsSymbol",
    "JsTimeout",
    "JSON",
    "Object",
    "PromiseRaw",
    "ReadableStream",
    "This",
    "Unknown",
    "WriteableStream",
  ]
  .into()
});

fn get_register_ident(name: &str) -> Ident {
  let new_name = format!(
    "__napi_register__{}_{}",
    rm_raw_prefix(name),
    REGISTER_INDEX.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
  );
  Ident::new(&new_name, Span::call_site())
}

struct AnyIdent(Ident);

impl Parse for AnyIdent {
  fn parse(input: ParseStream) -> SynResult<Self> {
    input.step(|cursor| match cursor.ident() {
      Some((ident, remaining)) => Ok((AnyIdent(ident), remaining)),
      None => Err(cursor.error("expected an identifier")),
    })
  }
}

pub trait ConvertToAST {
  fn convert_to_ast(&mut self, opts: &BindgenAttrs) -> BindgenResult<Napi>;
}

pub trait ParseNapi {
  fn parse_napi(&mut self, tokens: &mut TokenStream, opts: &BindgenAttrs) -> BindgenResult<Napi>;
}

/// This function does a few things:
/// - parses the tokens for the given argument `p` to find the `#[napi(ts_arg_type = "MyType")]`
///   attribute and return the manually overridden type.
/// - If both the `ts_args_type` override and the `ts_arg_type` override are present, bail
///   since it should only allow one at a time.
/// - Bails if it finds the `#[napi...]` attribute but it has the wrong data.
/// - Removes the attribute from the output token stream so this
///   `pub fn add(u: u32, #[napi(ts_arg_type = "MyType")] f: String)`
///   `  `turns into
///   `pub fn add(u: u32, f: String)`
///   `  `otherwise it won't compile
fn find_ts_arg_type_and_remove_attribute(
  p: &mut PatType,
  ts_args_type: Option<&(&str, Span)>,
) -> BindgenResult<Option<String>> {
  let mut ts_type_attr: Option<(usize, String)> = None;
  for (idx, attr) in p.attrs.iter().enumerate() {
    if attr.path().is_ident("napi") {
      if let Some((ts_args_type, _)) = ts_args_type {
        bail_span!(
          attr,
          "Found a 'ts_args_type'=\"{}\" override. Cannot use 'ts_arg_type' at the same time since they are mutually exclusive.",
          ts_args_type
        );
      }

      match &attr.meta {
        syn::Meta::Path(_) | syn::Meta::NameValue(_) => {
          bail_span!(
            attr,
            "Expects an assignment #[napi(ts_arg_type = \"MyType\")]"
          )
        }
        syn::Meta::List(list) => {
          let mut found = false;
          list
            .parse_args_with(|tokens: &syn::parse::ParseBuffer<'_>| {
              // tokens:
              // #[napi(xxx, xxx=xxx)]
              //        ^^^^^^^^^^^^
              let list = tokens.parse_terminated(Meta::parse, Token![,])?;

              for meta in list {
                if meta.path().is_ident("ts_arg_type") {
                  match meta {
                    Meta::Path(_) | Meta::List(_) => {
                      return Err(syn::Error::new(
                        meta.path().span(),
                        "Expects an assignment (ts_arg_type = \"MyType\")",
                      ));
                    }
                    Meta::NameValue(name_value) => match name_value.value {
                      syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(str),
                        ..
                      }) => {
                        let value = str.value();
                        found = true;
                        ts_type_attr = Some((idx, value));
                      }
                      _ => {
                        return Err(syn::Error::new(
                          name_value.value.span(),
                          "Expects a string literal",
                        ));
                      }
                    },
                  }
                }
              }

              Ok(())
            })
            .map_err(Diagnostic::from)?;

          if !found {
            bail_span!(attr, "Expects a 'ts_arg_type'");
          }
        }
      }
    }
  }

  if let Some((idx, value)) = ts_type_attr {
    p.attrs.remove(idx);
    Ok(Some(value))
  } else {
    Ok(None)
  }
}

fn find_enum_value_and_remove_attribute(v: &mut syn::Variant) -> BindgenResult<Option<String>> {
  let mut name_attr: Option<(usize, String)> = None;
  for (idx, attr) in v.attrs.iter().enumerate() {
    if attr.path().is_ident("napi") {
      match &attr.meta {
        syn::Meta::Path(_) | syn::Meta::NameValue(_) => {
          bail_span!(
            attr,
            "Expects an assignment #[napi(value = \"enum-variant-value\")]"
          )
        }
        syn::Meta::List(list) => {
          let mut found = false;
          list
            .parse_args_with(|tokens: &syn::parse::ParseBuffer<'_>| {
              // tokens:
              // #[napi(xxx, xxx=xxx)]
              //        ^^^^^^^^^^^^
              let list = tokens.parse_terminated(Meta::parse, Token![,])?;

              for meta in list {
                if meta.path().is_ident("value") {
                  match meta {
                    Meta::Path(_) | Meta::List(_) => {
                      return Err(syn::Error::new(
                        meta.path().span(),
                        "Expects an assignment (value = \"enum-variant-value\")",
                      ));
                    }
                    Meta::NameValue(name_value) => match name_value.value {
                      syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(str),
                        ..
                      }) => {
                        let value = str.value();
                        found = true;
                        name_attr = Some((idx, value));
                      }
                      _ => {
                        return Err(syn::Error::new(
                          name_value.value.span(),
                          "Expects a string literal",
                        ));
                      }
                    },
                  }
                }
              }

              Ok(())
            })
            .map_err(Diagnostic::from)?;

          if !found {
            bail_span!(attr, "Expects a 'value'");
          }
        }
      }
    }
  }

  if let Some((idx, value)) = name_attr {
    v.attrs.remove(idx);
    Ok(Some(value))
  } else {
    Ok(None)
  }
}

fn get_ty(mut ty: &mut syn::Type) -> &mut syn::Type {
  while let syn::Type::Group(g) = ty {
    ty = &mut g.elem;
  }

  ty
}

fn replace_self(mut ty: syn::Type, self_ty: Option<&Ident>) -> syn::Type {
  let self_ty = match self_ty {
    Some(i) => i,
    None => return ty,
  };
  let path = match get_ty(&mut ty) {
    syn::Type::Path(syn::TypePath { qself: None, path }) => path.clone(),
    other => return other.clone(),
  };
  let new_path = if path.segments.len() == 1 && path.segments[0].ident == "Self" {
    self_ty.clone().into()
  } else {
    path
  };
  syn::Type::Path(syn::TypePath {
    qself: None,
    path: new_path,
  })
}

/// Extracts the last ident from the path
fn extract_path_ident(path: &mut syn::Path) -> BindgenResult<(Ident, bool)> {
  let mut has_lifetime = false;
  for segment in path.segments.iter_mut() {
    match &segment.arguments {
      syn::PathArguments::None => {}
      syn::PathArguments::AngleBracketed(generic) => {
        if let Some(GenericArgument::Lifetime(_)) = generic.args.first() {
          has_lifetime = true;
        } else {
          bail_span!(path, "Only 1 lifetime is supported for now");
        }
      }
      _ => bail_span!(path, "paths with type parameters are not supported yet"),
    }
  }

  match path.segments.last() {
    Some(value) => Ok((value.ident.clone(), has_lifetime)),
    None => {
      bail_span!(path, "empty idents are not supported");
    }
  }
}

fn extract_callback_trait_types(
  arguments: &syn::PathArguments,
) -> BindgenResult<(Vec<syn::Type>, Option<syn::Type>)> {
  match arguments {
    // <T: Fn>
    syn::PathArguments::None => Ok((vec![], None)),
    syn::PathArguments::AngleBracketed(_) => {
      bail_span!(arguments, "use parentheses for napi callback trait")
    }
    syn::PathArguments::Parenthesized(arguments) => {
      let args = arguments.inputs.iter().cloned().collect::<Vec<_>>();

      let ret = match &arguments.output {
        syn::ReturnType::Type(_, ret_ty) => {
          let ret_ty = &**ret_ty;
          if let Some(ty_of_result) = extract_result_ty(ret_ty)? {
            if ty_of_result.to_token_stream().to_string() == "()" {
              None
            } else {
              Some(ty_of_result)
            }
          } else {
            bail_span!(ret_ty, "The return type of callback can only be `Result`");
          }
        }
        _ => {
          bail_span!(
            arguments,
            "The return type of callback can only be `Result`. Try with `Result<()>`"
          );
        }
      };

      Ok((args, ret))
    }
  }
}

fn extract_result_ty(ty: &syn::Type) -> BindgenResult<Option<syn::Type>> {
  match ty {
    syn::Type::Path(syn::TypePath { qself: None, path }) => {
      let segment = path.segments.last().unwrap();
      if segment.ident != "Result" {
        Ok(None)
      } else {
        match &segment.arguments {
          syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
            args, ..
          }) => {
            let ok_arg = args.first().unwrap();
            match ok_arg {
              syn::GenericArgument::Type(ty) => Ok(Some(ty.clone())),
              _ => bail_span!(ok_arg, "unsupported generic type"),
            }
          }
          _ => {
            bail_span!(segment, "unsupported generic type")
          }
        }
      }
    }
    _ => Ok(None),
  }
}

fn get_expr(mut expr: &syn::Expr) -> &syn::Expr {
  while let syn::Expr::Group(g) = expr {
    expr = &g.expr;
  }

  expr
}

/// Extract the documentation comments from a Vec of attributes
fn extract_doc_comments(attrs: &[syn::Attribute]) -> Vec<String> {
  attrs
    .iter()
    .filter_map(|a| {
      // if the path segments include an ident of "doc" we know this
      // this is a doc comment
      let name_value = a.meta.require_name_value();
      if let Ok(name) = name_value {
        if a.path().is_ident("doc") {
          Some(
            // We want to filter out any Puncts so just grab the Literals
            match &name.value {
              syn::Expr::Lit(ExprLit {
                lit: syn::Lit::Str(str),
                ..
              }) => {
                let quoted = str.token().to_string();
                Some(try_unescape(&quoted).unwrap_or(quoted))
              }
              _ => None,
            },
          )
        } else {
          None
        }
      } else {
        None
      }
    })
    //Fold up the [[String]] iter we created into Vec<String>
    .fold(vec![], |mut acc, a| {
      acc.extend(a);
      acc
    })
}

// Unescaped a quoted string. char::escape_debug() was used to escape the text.
fn try_unescape(s: &str) -> Option<String> {
  if s.is_empty() {
    return Some(String::new());
  }
  let mut result = String::with_capacity(s.len());
  let mut chars = s.chars();
  for i in 0.. {
    let c = match chars.next() {
      Some(c) => c,
      None => {
        if result.ends_with('"') {
          result.pop();
        }
        return Some(result);
      }
    };
    if i == 0 && c == '"' {
      // ignore it
    } else if c == '\\' {
      let c = chars.next()?;
      match c {
        't' => result.push('\t'),
        'r' => result.push('\r'),
        'n' => result.push('\n'),
        '\\' | '\'' | '"' => result.push(c),
        'u' => {
          if chars.next() != Some('{') {
            return None;
          }
          let (c, next) = unescape_unicode(&mut chars)?;
          result.push(c);
          if next != '}' {
            return None;
          }
        }
        _ => return None,
      }
    } else {
      result.push(c);
    }
  }
  None
}

fn unescape_unicode(chars: &mut Chars) -> Option<(char, char)> {
  let mut value = 0;
  for i in 0..7 {
    let c = chars.next()?;
    let num = match c {
      '0'..='9' => c as u32 - '0' as u32,
      'a'..='f' => c as u32 - 'a' as u32,
      'A'..='F' => c as u32 - 'A' as u32,
      _ => {
        if i == 0 {
          return None;
        }

        if i == 0 {
          return None;
        }
        let decoded = char::from_u32(value)?;
        return Some((decoded, c));
      }
    };

    if i >= 6 {
      return None;
    }
    value = (value << 4) | num;
  }
  None
}

fn extract_fn_closure_generics(
  generics: &syn::Generics,
) -> BindgenResult<HashMap<String, syn::PathArguments>> {
  let mut errors = vec![];

  let mut map = HashMap::default();
  if generics.params.is_empty() {
    return Ok(map);
  }

  if let Some(where_clause) = &generics.where_clause {
    for prediction in where_clause.predicates.iter() {
      match prediction {
        syn::WherePredicate::Type(syn::PredicateType {
          bounded_ty, bounds, ..
        }) => {
          for bound in bounds {
            match bound {
              syn::TypeParamBound::Trait(t) => {
                for segment in t.path.segments.iter() {
                  match segment.ident.to_string().as_str() {
                    "Fn" | "FnOnce" | "FnMut" => {
                      map.insert(
                        bounded_ty.to_token_stream().to_string(),
                        segment.arguments.clone(),
                      );
                    }
                    _ => {}
                  };
                }
              }
              syn::TypeParamBound::Lifetime(lifetime) => {
                if lifetime.ident != "static" {
                  errors.push(err_span!(
                    bound,
                    "only 'static is supported in lifetime bound for fn arguments"
                  ));
                }
              }
              _ => errors.push(err_span! {
                bound,
                "unsupported bound in napi"
              }),
            }
          }
        }
        _ => errors.push(err_span! {
          prediction,
          "unsupported where clause prediction in napi"
        }),
      };
    }
  }

  for param in generics.params.iter() {
    match param {
      syn::GenericParam::Type(syn::TypeParam { ident, bounds, .. }) => {
        for bound in bounds {
          match bound {
            syn::TypeParamBound::Trait(t) => {
              for segment in t.path.segments.iter() {
                match segment.ident.to_string().as_str() {
                  "Fn" | "FnOnce" | "FnMut" => {
                    map.insert(ident.to_string(), segment.arguments.clone());
                  }
                  _ => {}
                };
              }
            }
            syn::TypeParamBound::Lifetime(lifetime) => {
              if lifetime.ident != "static" {
                errors.push(err_span!(
                  bound,
                  "only 'static is supported in lifetime bound for fn arguments"
                ));
              }
            }
            _ => errors.push(err_span! {
              bound,
              "unsupported bound in napi"
            }),
          }
        }
      }
      syn::GenericParam::Lifetime(_) => {}
      _ => {
        errors.push(err_span!(param, "unsupported napi generic param for fn"));
      }
    }
  }

  Diagnostic::from_vec(errors).and(Ok(map))
}

fn napi_fn_from_decl(
  sig: &mut Signature,
  opts: &BindgenAttrs,
  attrs: Vec<Attribute>,
  vis: Visibility,
  parent: Option<&Ident>,
  parent_js_name: Option<String>,
) -> BindgenResult<NapiFn> {
  let mut errors = vec![];

  let syn::Signature {
    ident,
    asyncness,
    output,
    generics,
    ..
  } = sig.clone();

  let mut fn_self = None;
  let callback_traits = extract_fn_closure_generics(&generics)?;

  let args = sig
    .inputs
    .iter_mut()
    .filter_map(|arg| match arg {
      syn::FnArg::Typed(ref mut p) => {
        let ts_arg_type = find_ts_arg_type_and_remove_attribute(p, opts.ts_args_type().as_ref())
          .unwrap_or_else(|e| {
            errors.push(e);
            None
          });

        let ty_str = p.ty.to_token_stream().to_string();
        if let Some(path_arguments) = callback_traits.get(&ty_str) {
          match extract_callback_trait_types(path_arguments) {
            Ok((fn_args, fn_ret)) => Some(NapiFnArg {
              kind: NapiFnArgKind::Callback(Box::new(CallbackArg {
                pat: p.pat.clone(),
                args: fn_args,
                ret: fn_ret,
              })),
              ts_arg_type,
            }),
            Err(e) => {
              errors.push(e);
              None
            }
          }
        } else {
          let ty = replace_self(p.ty.as_ref().clone(), parent);
          *p.ty = ty;
          Some(NapiFnArg {
            kind: NapiFnArgKind::PatType(Box::new(p.clone())),
            ts_arg_type,
          })
        }
      }
      syn::FnArg::Receiver(r) => {
        if parent.is_some() {
          assert!(fn_self.is_none());
          if r.reference.is_none() {
            errors.push(err_span!(
              r,
              "The native methods can't move values from napi. Try `&self` or `&mut self` instead."
            ));
          } else if r.mutability.is_some() {
            fn_self = Some(FnSelf::MutRef);
          } else {
            fn_self = Some(FnSelf::Ref);
          }
        } else {
          errors.push(err_span!(r, "arguments cannot be `self`"));
        }
        None
      }
    })
    .collect::<Vec<_>>();

  let (ret, is_ret_result) = match output {
    syn::ReturnType::Default => (None, false),
    syn::ReturnType::Type(_, ty) => {
      let result_ty = extract_result_ty(&ty)?;
      if let Some(result_ty) = result_ty {
        (Some(replace_self(result_ty, parent)), true)
      } else {
        (Some(replace_self(*ty, parent)), false)
      }
    }
  };

  Diagnostic::from_vec(errors).and_then(|_| {
    let js_name = if let Some(prop_name) = opts.getter() {
      opts.js_name().map_or_else(
        || {
          if let Some(ident) = prop_name {
            ident.to_string()
          } else {
            to_case(ident.to_string().trim_start_matches("get_"), Case::Camel)
          }
        },
        |(js_name, _)| js_name.to_owned(),
      )
    } else if let Some(prop_name) = opts.setter() {
      opts.js_name().map_or_else(
        || {
          if let Some(ident) = prop_name {
            ident.to_string()
          } else {
            to_case(ident.to_string().trim_start_matches("set_"), Case::Camel)
          }
        },
        |(js_name, _)| js_name.to_owned(),
      )
    } else if opts.constructor().is_some() {
      "constructor".to_owned()
    } else if opts.module_exports().is_some() {
      if opts.js_name().is_some() {
        bail_span!(sig.ident, "module_exports fn can't have js_name");
      }
      if opts.getter().is_some() || opts.setter().is_some() {
        bail_span!(sig.ident, "module_exports fn can't have getter or setter");
      }
      if opts.factory().is_some() || opts.constructor().is_some() {
        bail_span!(
          sig.ident,
          "module_exports fn can't have factory or constructor"
        );
      }
      if opts.strict().is_some() {
        bail_span!(sig.ident, "module_exports fn can't have strict");
      }
      if opts.return_if_invalid().is_some() {
        bail_span!(sig.ident, "module_exports fn can't have return_if_invalid");
      }

      if parent.is_some() {
        bail_span!(sig.ident, "module_exports fn can't inside impl block");
      }

      if !generics.params.is_empty() {
        bail_span!(sig.ident, "module_exports fn can't have generic parameters");
      }

      if opts.no_export().is_some() {
        bail_span!(
          sig.ident,
          "#[napi(no_export)] can not be used with module_exports attribute"
        );
      }

      for arg in args.iter() {
        match &arg.kind {
          NapiFnArgKind::Callback(_) => {
            bail_span!(sig.ident, "module_exports fn can't have callback arguments");
          }
          NapiFnArgKind::PatType(pat) => {
            if arg.ts_arg_type.is_some() {
              bail_span!(sig.ident, "module_exports fn can't have ts_arg_type");
            }
            if let syn::Type::Path(syn::TypePath {
              path: syn::Path { segments, .. },
              ..
            }) = &*pat.ty
            {
              if let Some(segment) = segments.last() {
                if segment.ident != "Env" && segment.ident != "Object" {
                  bail_span!(
                    sig.ident,
                    "module_exports fn can only accept Env or Object as argument"
                  );
                }
                continue;
              }
            }
            if let syn::Type::Reference(syn::TypeReference { elem, .. }) = &*pat.ty {
              if let syn::Type::Path(syn::TypePath {
                path: syn::Path { segments, .. },
                ..
              }) = &**elem
              {
                if let Some(segment) = segments.last() {
                  if segment.ident != "Env" && segment.ident != "Object" {
                    bail_span!(
                      sig.ident,
                      "module_exports fn can only accept Env or Object as argument"
                    );
                  }
                  continue;
                }
              }
            }
          }
        }
        bail_span!(
          sig.ident,
          "module_exports fn can only accept Env or Object as argument"
        );
      }

      if let syn::ReturnType::Type(_, ty) = &sig.output {
        if let syn::Type::Path(syn::TypePath {
          path: syn::Path { segments, .. },
          ..
        }) = &**ty
        {
          if let Some(segment) = segments.last() {
            if segment.ident != "Result" && segment.ident != "()" {
              bail_span!(
                sig.ident,
                "module_exports fn can only return Result<()> or (), got {}",
                segment.ident
              );
            }
            if segment.ident == "Result" {
              if let syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
                args,
                ..
              }) = &segment.arguments
              {
                if args.len() != 1 {
                  bail_span!(
                    segment.ident,
                    "module_exports fn can only return Result<()> or ()"
                  );
                }
                if let syn::GenericArgument::Type(syn::Type::Tuple(syn::TypeTuple {
                  elems, ..
                })) = &args[0]
                {
                  if !elems.empty_or_trailing() {
                    bail_span!(
                      segment.ident,
                      "module_exports fn can only return Result<()> or ()"
                    );
                  }
                } else {
                  bail_span!(
                    segment.ident,
                    "module_exports fn can only return Result<()> or ()"
                  );
                }
              } else {
                bail_span!(
                  segment.ident,
                  "module_exports fn can only return Result<()> or ()"
                );
              }
            }
          }
        }
      }

      to_case(ident.to_string(), Case::Camel)
    } else {
      opts.js_name().map_or_else(
        || to_case(ident.to_string(), Case::Camel),
        |(js_name, _)| js_name.to_owned(),
      )
    };

    let namespace = opts.namespace().map(|(m, _)| m.to_owned());
    let (parent_is_generator, parent_is_async_generator) = if let Some(p) = parent {
      let generator_struct = GENERATOR_STRUCT.get_or_init(|| Mutex::new(HashMap::new()));
      let generator_struct = generator_struct
        .lock()
        .expect("Lock generator struct failed");

      let key = namespace
        .as_ref()
        .map(|n| format!("{n}::{p}"))
        .unwrap_or_else(|| p.to_string());
      *generator_struct.get(&key).unwrap_or(&(false, false))
    } else {
      (false, false)
    };

    let kind = fn_kind(opts);

    if !matches!(kind, FnKind::Normal) && parent.is_none() {
      bail_span!(
        sig.ident,
        "Only fn in impl block can be marked as factory, constructor, getter or setter"
      );
    }

    if matches!(kind, FnKind::Constructor) && asyncness.is_some() {
      bail_span!(sig.ident, "Constructor don't support asynchronous function");
    }

    Ok(NapiFn {
      name: ident.clone(),
      js_name,
      module_exports: opts.module_exports().is_some(),
      args,
      ret,
      is_ret_result,
      is_async: asyncness.is_some(),
      within_async_runtime: opts.async_runtime().is_some(),
      vis,
      kind,
      fn_self,
      parent: parent.cloned(),
      parent_js_name,
      comments: extract_doc_comments(&attrs),
      attrs,
      strict: opts.strict().is_some(),
      return_if_invalid: opts.return_if_invalid().is_some(),
      js_mod: opts.namespace().map(|(m, _)| m.to_owned()),
      ts_type: opts.ts_type().map(|(m, _)| m.to_owned()),
      ts_generic_types: opts.ts_generic_types().map(|(m, _)| m.to_owned()),
      ts_args_type: opts.ts_args_type().map(|(m, _)| m.to_owned()),
      ts_return_type: opts.ts_return_type().map(|(m, _)| m.to_owned()),
      skip_typescript: opts.skip_typescript().is_some(),
      parent_is_generator,
      parent_is_async_generator,
      writable: opts.writable(),
      enumerable: opts.enumerable(),
      configurable: opts.configurable(),
      catch_unwind: opts.catch_unwind().is_some(),
      unsafe_: sig.unsafety.is_some(),
      register_name: get_register_ident(ident.to_string().as_str()),
      no_export: opts.no_export().is_some(),
    })
  })
}

impl ParseNapi for syn::Item {
  fn parse_napi(&mut self, tokens: &mut TokenStream, opts: &BindgenAttrs) -> BindgenResult<Napi> {
    match self {
      syn::Item::Fn(f) => f.parse_napi(tokens, opts),
      syn::Item::Struct(s) => s.parse_napi(tokens, opts),
      syn::Item::Impl(i) => i.parse_napi(tokens, opts),
      syn::Item::Enum(e) => e.parse_napi(tokens, opts),
      syn::Item::Const(c) => c.parse_napi(tokens, opts),
      syn::Item::Type(c) => c.parse_napi(tokens, opts),
      _ => bail_span!(
        self,
        "#[napi] can only be applied to a function, struct, enum, const, mod or impl."
      ),
    }
  }
}

impl ParseNapi for syn::ItemFn {
  fn parse_napi(&mut self, tokens: &mut TokenStream, opts: &BindgenAttrs) -> BindgenResult<Napi> {
    if opts.ts_type().is_some()
      && (opts.ts_args_type().is_some() || opts.ts_return_type().is_some())
    {
      bail_span!(
        self,
        "#[napi] with ts_type cannot be combined with ts_args_type, ts_return_type in function"
      );
    }
    if opts.return_if_invalid().is_some() && opts.strict().is_some() {
      bail_span!(
        self,
        "#[napi(return_if_invalid)] can't be used with #[napi(strict)]"
      );
    }
    let napi = self.convert_to_ast(opts);
    self.to_tokens(tokens);

    napi
  }
}
impl ParseNapi for syn::ItemStruct {
  fn parse_napi(&mut self, tokens: &mut TokenStream, opts: &BindgenAttrs) -> BindgenResult<Napi> {
    if opts.ts_args_type().is_some()
      || opts.ts_return_type().is_some()
      || opts.skip_typescript().is_some()
      || opts.ts_type().is_some()
    {
      bail_span!(
        self,
        "#[napi] can't be applied to a struct with #[napi(ts_args_type)], #[napi(ts_return_type)], #[napi(skip_typescript)] or #[napi(ts_type)]"
      );
    }
    if opts.return_if_invalid().is_some() {
      bail_span!(
        self,
        "#[napi(return_if_invalid)] can only be applied to a function or method."
      );
    }
    if opts.catch_unwind().is_some() {
      bail_span!(
        self,
        "#[napi(catch_unwind)] can only be applied to a function or method."
      );
    }
    if opts.no_export().is_some() {
      bail_span!(
        self,
        "#[napi(no_export)] can only be applied to a function."
      );
    }
    if opts.object().is_some() && opts.custom_finalize().is_some() {
      bail_span!(self, "Custom finalize is not supported for #[napi(object)]");
    }
    let napi = self.convert_to_ast(opts);
    self.to_tokens(tokens);

    napi
  }
}

impl ParseNapi for syn::ItemImpl {
  fn parse_napi(&mut self, tokens: &mut TokenStream, opts: &BindgenAttrs) -> BindgenResult<Napi> {
    if opts.ts_args_type().is_some()
      || opts.ts_return_type().is_some()
      || opts.skip_typescript().is_some()
      || opts.ts_type().is_some()
      || opts.custom_finalize().is_some()
    {
      bail_span!(
        self,
        "#[napi] can't be applied to impl with #[napi(ts_args_type)], #[napi(ts_return_type)], #[napi(skip_typescript)] or #[napi(ts_type)] or #[napi(custom_finalize)]"
      );
    }
    if opts.return_if_invalid().is_some() {
      bail_span!(
        self,
        "#[napi(return_if_invalid)] can only be applied to a function or method."
      );
    }
    if opts.catch_unwind().is_some() {
      bail_span!(
        self,
        "#[napi(catch_unwind)] can only be applied to a function or method."
      );
    }
    if opts.no_export().is_some() {
      bail_span!(
        self,
        "#[napi(no_export)] can only be applied to a function."
      );
    }
    // #[napi] macro will be remove from impl items after converted to ast
    let napi = self.convert_to_ast(opts);
    self.to_tokens(tokens);

    napi
  }
}

impl ParseNapi for syn::ItemEnum {
  fn parse_napi(&mut self, tokens: &mut TokenStream, opts: &BindgenAttrs) -> BindgenResult<Napi> {
    if opts.ts_args_type().is_some()
      || opts.ts_return_type().is_some()
      || opts.ts_type().is_some()
      || opts.custom_finalize().is_some()
    {
      bail_span!(
        self,
        "#[napi] can't be applied to a enum with #[napi(ts_args_type)], #[napi(ts_return_type)] or #[napi(ts_type)] or #[napi(custom_finalize)]"
      );
    }
    if opts.return_if_invalid().is_some() {
      bail_span!(
        self,
        "#[napi(return_if_invalid)] can only be applied to a function or method."
      );
    }
    if opts.catch_unwind().is_some() {
      bail_span!(
        self,
        "#[napi(catch_unwind)] can only be applied to a function or method."
      );
    }
    if opts.no_export().is_some() {
      bail_span!(
        self,
        "#[napi(no_export)] can only be applied to a function."
      );
    }
    let napi = self.convert_to_ast(opts);
    self.to_tokens(tokens);

    napi
  }
}
impl ParseNapi for syn::ItemConst {
  fn parse_napi(&mut self, tokens: &mut TokenStream, opts: &BindgenAttrs) -> BindgenResult<Napi> {
    if opts.ts_args_type().is_some()
      || opts.ts_return_type().is_some()
      || opts.ts_type().is_some()
      || opts.custom_finalize().is_some()
    {
      bail_span!(
        self,
        "#[napi] can't be applied to a const with #[napi(ts_args_type)], #[napi(ts_return_type)] or #[napi(ts_type)] or #[napi(custom_finalize)]"
      );
    }
    if opts.return_if_invalid().is_some() {
      bail_span!(
        self,
        "#[napi(return_if_invalid)] can only be applied to a function or method."
      );
    }
    if opts.catch_unwind().is_some() {
      bail_span!(
        self,
        "#[napi(catch_unwind)] can only be applied to a function or method."
      );
    }
    if opts.no_export().is_some() {
      bail_span!(
        self,
        "#[napi(no_export)] can only be applied to a function."
      );
    }
    let napi = self.convert_to_ast(opts);
    self.to_tokens(tokens);
    napi
  }
}

impl ParseNapi for syn::ItemType {
  fn parse_napi(&mut self, tokens: &mut TokenStream, opts: &BindgenAttrs) -> BindgenResult<Napi> {
    if opts.ts_args_type().is_some()
      || opts.ts_return_type().is_some()
      || opts.custom_finalize().is_some()
    {
      bail_span!(
        self,
        "#[napi] can't be applied to a type with #[napi(ts_args_type)], #[napi(ts_return_type)] or #[napi(custom_finalize)]"
      );
    }
    if opts.return_if_invalid().is_some() {
      bail_span!(
        self,
        "#[napi(return_if_invalid)] can only be applied to a function or method."
      );
    }
    if opts.catch_unwind().is_some() {
      bail_span!(
        self,
        "#[napi(catch_unwind)] can only be applied to a function or method."
      );
    }
    if opts.no_export().is_some() {
      bail_span!(
        self,
        "#[napi(no_export)] can only be applied to a function."
      );
    }
    let napi = self.convert_to_ast(opts);
    self.to_tokens(tokens);
    napi
  }
}

fn fn_kind(opts: &BindgenAttrs) -> FnKind {
  let mut kind = FnKind::Normal;

  if opts.getter().is_some() {
    kind = FnKind::Getter;
  }

  if opts.setter().is_some() {
    kind = FnKind::Setter;
  }

  if opts.constructor().is_some() {
    kind = FnKind::Constructor;
  }

  if opts.factory().is_some() {
    kind = FnKind::Factory;
  }

  kind
}

impl ConvertToAST for syn::ItemFn {
  fn convert_to_ast(&mut self, opts: &BindgenAttrs) -> BindgenResult<Napi> {
    let func = napi_fn_from_decl(
      &mut self.sig,
      opts,
      self.attrs.clone(),
      self.vis.clone(),
      None,
      None,
    )?;

    Ok(Napi {
      item: NapiItem::Fn(func),
    })
  }
}

fn convert_fields(
  fields: &mut syn::Fields,
  check_vis: bool,
) -> BindgenResult<(Vec<NapiStructField>, bool)> {
  let mut napi_fields = vec![];
  let is_tuple = matches!(fields, syn::Fields::Unnamed(_));
  for (i, field) in fields.iter_mut().enumerate() {
    if check_vis && !matches!(field.vis, syn::Visibility::Public(_)) {
      continue;
    }

    let field_opts = BindgenAttrs::find(&mut field.attrs)?;

    let (js_name, name) = match &field.ident {
      Some(ident) => (
        field_opts.js_name().map_or_else(
          || to_case(ident.unraw().to_string(), Case::Camel),
          |(js_name, _)| js_name.to_owned(),
        ),
        syn::Member::Named(ident.clone()),
      ),
      None => (
        field_opts
          .js_name()
          .map_or_else(|| format!("field{i}"), |(js_name, _)| js_name.to_owned()),
        syn::Member::Unnamed(i.into()),
      ),
    };

    let ignored = field_opts.skip().is_some();
    let readonly = field_opts.readonly().is_some();
    let writable = field_opts.writable();
    let enumerable = field_opts.enumerable();
    let configurable = field_opts.configurable();
    let skip_typescript = field_opts.skip_typescript().is_some();
    let ts_type = field_opts.ts_type().map(|e| e.0.to_string());

    let mut ty = field.ty.clone();

    let has_lifetime = if let Type::Path(syn::TypePath {
      path: Path { segments, .. },
      ..
    }) = &mut ty
    {
      if let Some(PathSegment {
        arguments: PathArguments::AngleBracketed(AngleBracketedGenericArguments { args, .. }),
        ..
      }) = segments.last_mut()
      {
        args.iter_mut().any(|arg| {
          if let GenericArgument::Lifetime(lifetime) = arg {
            *lifetime = syn::Lifetime::new("'static", Span::call_site());
            true
          } else {
            false
          }
        })
      } else {
        false
      }
    } else {
      false
    };

    napi_fields.push(NapiStructField {
      name,
      js_name,
      ty,
      getter: !ignored,
      setter: !(ignored || readonly),
      writable,
      enumerable,
      configurable,
      comments: extract_doc_comments(&field.attrs),
      skip_typescript,
      ts_type,
      has_lifetime,
    })
  }
  Ok((napi_fields, is_tuple))
}

impl ConvertToAST for syn::ItemStruct {
  fn convert_to_ast(&mut self, opts: &BindgenAttrs) -> BindgenResult<Napi> {
    let mut errors = vec![];

    let rust_struct_ident: Ident = self.ident.clone();
    let final_js_name_for_struct = opts.js_name().map_or_else(
      || to_case(self.ident.to_string(), Case::Pascal),
      |(attr_js_name, _span)| attr_js_name.to_owned(),
    );

    let use_nullable = opts.use_nullable();
    let (fields, is_tuple) = convert_fields(&mut self.fields, true)?;

    record_struct(&rust_struct_ident, final_js_name_for_struct.clone(), opts);
    let namespace = opts.namespace().map(|(m, _)| m.to_owned());
    let implement_iterator = opts.iterator().is_some();
    let implement_async_iterator = opts.async_iterator().is_some();

    if implement_iterator && implement_async_iterator {
      bail_span!(
        self,
        "Cannot use both #[napi(iterator)] and #[napi(async_iterator)] on the same struct. \
         Use #[napi(iterator)] for synchronous iteration (impl Generator) or \
         #[napi(async_iterator)] for async iteration (impl AsyncGenerator)"
      );
    }

    if (implement_iterator || implement_async_iterator)
      && self
        .fields
        .iter()
        .filter(|f| matches!(f.vis, Visibility::Public(_)))
        .filter_map(|f| f.ident.clone())
        .map(|ident| ident.to_string())
        .any(|field_name| field_name == "next" || field_name == "throw" || field_name == "return")
    {
      bail_span!(
        self,
        "Generator structs cannot have public fields named `next`, `throw`, or `return`."
      );
    }

    let generator_struct = GENERATOR_STRUCT.get_or_init(|| Mutex::new(HashMap::new()));
    let mut generator_struct = generator_struct
      .lock()
      .expect("Lock generator struct failed");
    let key = namespace
      .as_ref()
      .map(|n| format!("{n}::{rust_struct_ident}"))
      .unwrap_or_else(|| rust_struct_ident.to_string());
    generator_struct.insert(key, (implement_iterator, implement_async_iterator));
    drop(generator_struct);

    let transparent = opts
      .transparent()
      .is_some()
      .then(|| -> Result<_, Diagnostic> {
        if !is_tuple || self.fields.len() != 1 {
          bail_span!(
            self,
            "#[napi(transparent)] can only be applied to a struct with a single field tuple",
          )
        }
        let first_field = self.fields.iter().next().unwrap();
        Ok(first_field.ty.clone())
      })
      .transpose()?;

    let struct_kind = if let Some(transparent) = transparent {
      NapiStructKind::Transparent(NapiTransparent {
        ty: transparent,
        object_from_js: opts.object_from_js(),
        object_to_js: opts.object_to_js(),
      })
    } else if opts.array().is_some() {
      if !is_tuple {
        bail_span!(self, "#[napi(array)] can only be applied to a tuple struct",)
      }
      NapiStructKind::Array(NapiArray {
        fields,
        object_from_js: opts.object_from_js(),
        object_to_js: opts.object_to_js(),
      })
    } else if opts.object().is_some() {
      NapiStructKind::Object(NapiObject {
        fields,
        object_from_js: opts.object_from_js(),
        object_to_js: opts.object_to_js(),
        is_tuple,
      })
    } else {
      // field lifetime check, JsValue types with lifetime can't be assigned to a field of napi class struct
      for syn::Field { ty, .. } in self.fields.iter() {
        if let syn::Type::Path(syn::TypePath { path, .. }) = ty {
          if let Some(PathSegment {
            ident,
            arguments:
              syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments { args, .. }),
            ..
          }) = path.segments.last()
          {
            if let Some(GenericArgument::Lifetime(syn::Lifetime { ident: _, .. })) = args.first() {
              // has lifetime and type name matched with known js value types
              if KNOWN_JS_VALUE_TYPES_WITH_LIFETIME.contains(ident.to_string().as_str()) {
                // TODO: add link for more information
                errors.push(err_span!(
                  ty,
                  "Can't assign {} to a field of napi class struct",
                  ident
                ));
              }
            }
          }
        }
      }
      NapiStructKind::Class(NapiClass {
        fields,
        ctor: opts.constructor().is_some(),
        implement_iterator,
        implement_async_iterator,
        is_tuple,
        use_custom_finalize: opts.custom_finalize().is_some(),
      })
    };

    match &struct_kind {
      NapiStructKind::Transparent(_) => {}
      NapiStructKind::Class(class) if !class.ctor => {}
      _ => {
        for field in self.fields.iter() {
          if !matches!(field.vis, syn::Visibility::Public(_)) {
            errors.push(err_span!(
              field,
              "#[napi] requires all struct fields to be public to mark struct as constructor or object shape\nthis field is not public."
            ));
          }
        }
      }
    };

    if self.generics.lifetimes().size_hint().0 > 1 {
      errors.push(err_span!(
        self,
        "struct with multiple generic parameters is not supported"
      ));
    }

    let lifetime = if let Some(lifetime) = self.generics.lifetimes().next() {
      if !lifetime.bounds.is_empty() {
        bail_span!(lifetime.bounds, "unsupported self type in #[napi] impl")
      }
      Some(lifetime.lifetime.to_string())
    } else {
      None
    };

    Diagnostic::from_vec(errors).map(|()| Napi {
      item: NapiItem::Struct(NapiStruct {
        js_name: final_js_name_for_struct,
        name: rust_struct_ident.clone(),
        kind: struct_kind,
        js_mod: namespace,
        use_nullable,
        register_name: get_register_ident(format!("{rust_struct_ident}_struct").as_str()),
        comments: extract_doc_comments(&self.attrs),
        has_lifetime: lifetime.is_some(),
        is_generator: implement_iterator,
        is_async_generator: implement_async_iterator,
      }),
    })
  }
}

impl ConvertToAST for syn::ItemImpl {
  fn convert_to_ast(&mut self, impl_opts: &BindgenAttrs) -> BindgenResult<Napi> {
    let struct_name = match get_ty(&mut self.self_ty) {
      syn::Type::Path(syn::TypePath {
        ref mut path,
        qself: None,
      }) => path,
      _ => {
        bail_span!(self.self_ty, "unsupported self type in #[napi] impl")
      }
    };

    let (struct_name, has_lifetime) = extract_path_ident(struct_name)?;

    // Check if this struct was recorded with a custom js_name, fallback to default if not found
    let mut struct_js_name =
      match check_recorded_struct_for_impl(&struct_name, &BindgenAttrs::default()) {
        Ok(recorded_js_name) => recorded_js_name,
        Err(_) => to_case(struct_name.to_string(), Case::UpperCamel),
      };
    let mut items = vec![];
    let mut task_output_type = None;
    let mut iterator_yield_type = None;
    let mut iterator_next_type = None;
    let mut iterator_return_type = None;
    let mut async_iterator_yield_type = None;
    let mut async_iterator_next_type = None;
    let mut async_iterator_return_type = None;
    for item in self.items.iter_mut() {
      if let Some(method) = match item {
        syn::ImplItem::Fn(m) => Some(m),
        syn::ImplItem::Type(m) => {
          if let Some((_, t, _)) = &self.trait_ {
            if let Some(PathSegment { ident, .. }) = t.segments.last() {
              if (ident == "Task" || ident == "ScopedTask") && m.ident == "JsValue" {
                task_output_type = Some(m.ty.clone());
              } else if ident == "Generator" || ident == "ScopedGenerator" {
                if let Type::Path(_) = &m.ty {
                  if m.ident == "Yield" {
                    iterator_yield_type = Some(m.ty.clone());
                  } else if m.ident == "Next" {
                    iterator_next_type = Some(m.ty.clone());
                  } else if m.ident == "Return" {
                    iterator_return_type = Some(m.ty.clone());
                  }
                }
              } else if ident == "AsyncGenerator" {
                if let Type::Path(_) = &m.ty {
                  if m.ident == "Yield" {
                    async_iterator_yield_type = Some(m.ty.clone());
                  } else if m.ident == "Next" {
                    async_iterator_next_type = Some(m.ty.clone());
                  } else if m.ident == "Return" {
                    async_iterator_return_type = Some(m.ty.clone());
                  }
                }
              }
            }
          }
          None
        }
        _ => {
          bail_span!(item, "unsupported impl item in #[napi]")
        }
      } {
        let opts = BindgenAttrs::find(&mut method.attrs)?;

        // it'd better only care methods decorated with `#[napi]` attribute
        if !opts.exists {
          continue;
        }

        if opts.constructor().is_some() || opts.factory().is_some() {
          struct_js_name = check_recorded_struct_for_impl(&struct_name, &opts)?;
        }

        let vis = method.vis.clone();

        match &vis {
          Visibility::Public(_) => {}
          _ => {
            bail_span!(method.sig.ident, "only pub method supported by #[napi].",);
          }
        }

        let func = napi_fn_from_decl(
          &mut method.sig,
          &opts,
          method.attrs.clone(),
          vis,
          Some(&struct_name),
          Some(struct_js_name.clone()),
        )?;

        items.push(func);
      }
    }

    let namespace = impl_opts.namespace().map(|(m, _)| m.to_owned());

    Ok(Napi {
      item: NapiItem::Impl(NapiImpl {
        name: struct_name.clone(),
        js_name: struct_js_name,
        items,
        task_output_type,
        iterator_yield_type,
        iterator_next_type,
        iterator_return_type,
        async_iterator_yield_type,
        async_iterator_next_type,
        async_iterator_return_type,
        has_lifetime,
        js_mod: namespace,
        comments: extract_doc_comments(&self.attrs),
        register_name: get_register_ident(format!("{struct_name}_impl").as_str()),
      }),
    })
  }
}

impl ConvertToAST for syn::ItemEnum {
  fn convert_to_ast(&mut self, opts: &BindgenAttrs) -> BindgenResult<Napi> {
    match self.vis {
      Visibility::Public(_) => {}
      _ => bail_span!(self, "only public enum allowed"),
    }

    let js_name = opts
      .js_name()
      .map_or_else(|| self.ident.to_string(), |(s, _)| s.to_string());
    let is_string_enum = opts.string_enum().is_some();

    if self
      .variants
      .iter()
      .any(|v| !matches!(v.fields, syn::Fields::Unit))
    {
      let discriminant = opts.discriminant().map_or("type", |(s, _)| s);
      let discriminant_case = opts.discriminant_case().map(|c|
        Ok::<Case, Diagnostic>(match c.0 {
          "lowercase" => Case::Flat,
          "UPPERCASE" => Case::UpperFlat,
          "PascalCase" => Case::Pascal,
          "camelCase" => Case::Camel,
          "snake_case" => Case::Snake,
          "UPPER_SNAKE" => Case::UpperSnake,
          "kebab-case" => Case::Kebab,
          "UPPER-KEBAB-CASE" => Case::UpperKebab,
          _ => {
            bail_span!(self, "Unknown discriminant case. Possible values are \"lowercase\", \"UPPERCASE\", \"PascalCase\", \"camelCase\", \"snake_case\", \"UPPER_SNAKE\", \"kebab-case\", or \"UPPER-KEBAB-CASE\"")
          }
        })
      ).transpose()?;

      let mut errors = vec![];
      let mut variants = vec![];
      for variant in self.variants.iter_mut() {
        let (fields, is_tuple) = convert_fields(&mut variant.fields, false)?;
        for field in fields.iter() {
          if field.js_name == discriminant {
            errors.push(err_span!(
              field.name,
              r#"field's js_name("{}") and discriminator("{}") conflict"#,
              field.js_name,
              discriminant,
            ));
          }
        }
        variants.push(NapiStructuredEnumVariant {
          name: variant.ident.clone(),
          fields,
          is_tuple,
        });
      }
      let rust_struct_ident = self.ident.clone();
      return Diagnostic::from_vec(errors).map(|()| Napi {
        item: NapiItem::Struct(NapiStruct {
          name: rust_struct_ident.clone(),
          js_name,
          comments: extract_doc_comments(&self.attrs),
          js_mod: opts.namespace().map(|(m, _)| m.to_owned()),
          use_nullable: opts.use_nullable(),
          register_name: get_register_ident(format!("{rust_struct_ident}_struct").as_str()),
          kind: NapiStructKind::StructuredEnum(NapiStructuredEnum {
            variants,
            discriminant: discriminant.to_owned(),
            discriminant_case,
            object_from_js: opts.object_from_js(),
            object_to_js: opts.object_to_js(),
          }),
          has_lifetime: false,
          is_generator: false,
          is_async_generator: false,
        }),
      });
    }

    let variants = match opts.string_enum() {
      Some(case) => {
        let case = case.map(|c| Ok::<Case, Diagnostic>(match c.0.as_str() {
          "lowercase" => Case::Flat,
          "UPPERCASE" => Case::UpperFlat,
          "PascalCase" => Case::Pascal,
          "camelCase" => Case::Camel,
          "snake_case" => Case::Snake,
          "UPPER_SNAKE" => Case::UpperSnake,
          "kebab-case" => Case::Kebab,
          "UPPER-KEBAB-CASE" => Case::UpperKebab,
          _ => {
            bail_span!(self, "Unknown string enum case. Possible values are \"lowercase\", \"UPPERCASE\", \"PascalCase\", \"camelCase\", \"snake_case\", \"UPPER_SNAKE\", \"kebab-case\", or \"UPPER-KEBAB-CASE\"")
          }
        })).transpose()?;

        self
          .variants
          .iter_mut()
          .map(|v| {
            if !matches!(v.fields, syn::Fields::Unit) {
              bail_span!(
                v.fields,
                "Structured enum is not supported with string enum in #[napi]"
              )
            }
            if matches!(&v.discriminant, Some((_, _))) {
              bail_span!(
                v.fields,
                "Literal values are not supported with string enum in #[napi]"
              )
            }

            let val = find_enum_value_and_remove_attribute(v)?.unwrap_or_else(|| {
              let mut val = v.ident.to_string();
              if let Some(case) = case {
                val = to_case(val, case)
              }
              val
            });

            Ok(NapiEnumVariant {
              name: v.ident.clone(),
              val: NapiEnumValue::String(val),
              comments: extract_doc_comments(&v.attrs),
            })
          })
          .collect::<BindgenResult<Vec<NapiEnumVariant>>>()?
      }
      None => {
        let mut last_variant_val: i32 = -1;

        self
          .variants
          .iter()
          .map(|v| {
            let val = match &v.discriminant {
              Some((_, expr)) => {
                let mut symbol = 1;
                let mut inner_expr = get_expr(expr);
                if let syn::Expr::Unary(syn::ExprUnary {
                  attrs: _,
                  op: syn::UnOp::Neg(_),
                  expr,
                }) = inner_expr
                {
                  symbol = -1;
                  inner_expr = expr;
                }

                match inner_expr {
                  syn::Expr::Lit(syn::ExprLit {
                    attrs: _,
                    lit: syn::Lit::Int(int_lit),
                  }) => match int_lit.base10_digits().parse::<i32>() {
                    Ok(v) => symbol * v,
                    Err(_) => {
                      bail_span!(
                        int_lit,
                        "enums with #[wasm_bindgen] can only support \
                      numbers that can be represented as i32",
                      );
                    }
                  },
                  _ => bail_span!(
                    expr,
                    "enums with #[wasm_bindgen] may only have \
                  number literal values",
                  ),
                }
              }
              None => last_variant_val + 1,
            };

            last_variant_val = val;

            Ok(NapiEnumVariant {
              name: v.ident.clone(),
              val: NapiEnumValue::Number(val),
              comments: extract_doc_comments(&v.attrs),
            })
          })
          .collect::<BindgenResult<Vec<NapiEnumVariant>>>()?
      }
    };

    Ok(Napi {
      item: NapiItem::Enum(NapiEnum {
        name: self.ident.clone(),
        js_name,
        variants,
        js_mod: opts.namespace().map(|(m, _)| m.to_owned()),
        comments: extract_doc_comments(&self.attrs),
        skip_typescript: opts.skip_typescript().is_some(),
        register_name: get_register_ident(self.ident.to_string().as_str()),
        is_string_enum,
        object_from_js: opts.object_from_js(),
        object_to_js: opts.object_to_js(),
      }),
    })
  }
}

impl ConvertToAST for syn::ItemConst {
  fn convert_to_ast(&mut self, opts: &BindgenAttrs) -> BindgenResult<Napi> {
    match self.vis {
      Visibility::Public(_) => Ok(Napi {
        item: NapiItem::Const(NapiConst {
          name: self.ident.clone(),
          js_name: opts
            .js_name()
            .map_or_else(|| self.ident.to_string(), |(s, _)| s.to_string()),
          type_name: *self.ty.clone(),
          value: *self.expr.clone(),
          js_mod: opts.namespace().map(|(m, _)| m.to_owned()),
          comments: extract_doc_comments(&self.attrs),
          skip_typescript: opts.skip_typescript().is_some(),
          register_name: get_register_ident(self.ident.to_string().as_str()),
        }),
      }),
      _ => bail_span!(self, "only public const allowed"),
    }
  }
}

impl ConvertToAST for syn::ItemType {
  fn convert_to_ast(&mut self, opts: &BindgenAttrs) -> BindgenResult<Napi> {
    let js_name = match opts.js_name() {
      Some((name, _)) => name.to_string(),
      _ => {
        let types = self
          .generics
          .type_params()
          .map(|param| param.ident.to_string())
          .collect::<Vec<String>>()
          .join(", ");

        if !types.is_empty() {
          format!("{}<{}>", self.ident, types)
        } else {
          self.ident.to_string()
        }
      }
    };

    match self.vis {
      Visibility::Public(_) => Ok(Napi {
        item: NapiItem::Type(NapiType {
          name: self.ident.clone(),
          js_name,
          value: *self.ty.clone(),
          js_mod: opts.namespace().map(|(m, _)| m.to_owned()),
          comments: extract_doc_comments(&self.attrs),
          skip_typescript: opts.skip_typescript().is_some(),
          register_name: get_register_ident(self.ident.to_string().as_str()),
        }),
      }),
      _ => bail_span!(self, "only public type allowed"),
    }
  }
}
