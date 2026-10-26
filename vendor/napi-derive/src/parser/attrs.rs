use std::{
  cell::Cell,
  collections::HashMap,
  sync::{
    Mutex, OnceLock,
    atomic::{AtomicUsize, Ordering},
  },
};

use napi_derive_backend::{BindgenResult, Diagnostic, bail_span};
use proc_macro2::{Delimiter, Ident, Span, TokenTree};
use quote::ToTokens;
use syn::{
  Attribute, Token,
  parse::{Parse, ParseStream},
  parse_quote,
  spanned::Spanned,
};

use crate::parser::AnyIdent;

static ATTRS: OnceLock<AttributeParseState> = OnceLock::new();
static STRUCTS: OnceLock<StructParseState> = OnceLock::new();

#[derive(Default)]
struct StructParseState {
  parsed: Mutex<HashMap<String, ParsedStruct>>,
}

struct ParsedStruct {
  js_name: String,
  ctor_defined: bool,
}

#[derive(Default)]
struct AttributeParseState {
  parsed: AtomicUsize,
  #[allow(unused)]
  checks: AtomicUsize,
}

#[derive(Debug)]
/// Parsed attributes from a `#[napi(..)]`.
pub struct BindgenAttrs {
  /// Whether `#[napi]` attribute exists
  pub exists: bool,
  /// List of parsed attributes
  pub attrs: Vec<(Cell<bool>, BindgenAttr)>,
  /// Span of original attribute
  pub span: Span,
}

// NOTE: borrowed from wasm-bindgen
// some of them may useless in #[napi] macro
macro_rules! attrgen {
  ($mac:ident) => {
    $mac! {
      (catch_unwind, CatchUnwind(Span)),
      (async_runtime, AsyncRuntime(Span)),
      (module_exports, ModuleExports(Span)),
      (js_name, JsName(Span, String, Span)),
      (constructor, Constructor(Span)),
      (factory, Factory(Span)),
      (getter, Getter(Span, Option<Ident>)),
      (setter, Setter(Span, Option<Ident>)),
      (readonly, Readonly(Span)),
      (enumerable, Enumerable(Span, Option<bool>), true),
      (writable, Writable(Span, Option<bool>), true),
      (configurable, Configurable(Span, Option<bool>), true),
      (skip, Skip(Span)),
      (strict, Strict(Span)),
      (return_if_invalid, ReturnIfInvalid(Span)),
      (object, Object(Span)),
      (object_from_js, ObjectFromJs(Span, Option<bool>), true),
      (object_to_js, ObjectToJs(Span, Option<bool>), true),
      (custom_finalize, CustomFinalize(Span)),
      (namespace, Namespace(Span, String, Span)),
      (iterator, Iterator(Span)),
      (async_iterator, AsyncIterator(Span)),
      (ts_args_type, TsArgsType(Span, String, Span)),
      (ts_return_type, TsReturnType(Span, String, Span)),
      (ts_type, TsType(Span, String, Span)),
      (ts_generic_types, TsGenericTypes(Span, String, Span)),
      (string_enum, StringEnum(Span, Option<(String, Span)>)),
      (use_nullable, UseNullable(Span, Option<bool>), false),
      (discriminant, Discriminant(Span, String, Span)),
      (discriminant_case, DiscriminantCase(Span, String, Span)),
      (transparent, Transparent(Span)),
      (array, Array(Span)),
      (no_export, NoExport(Span)),

      // impl later
      // (inspectable, Inspectable(Span)),
      // (typescript_custom_section, TypescriptCustomSection(Span)),
      (skip_typescript, SkipTypescript(Span)),
      // (getter_with_clone, GetterWithClone(Span)),

      // For testing purposes only.
      // (assert_no_shim, AssertNoShim(Span)),
    }
  };
}

macro_rules! methods {
  ($(($name:ident, $variant:ident($($contents:tt)*) $($extra_tokens:tt)*),)*) => {
    $(methods!(@method $name, $variant($($contents)*) $($extra_tokens)*);)*

    #[cfg(feature = "strict")]
    #[allow(unused)]
    pub fn check_used(&self) -> Result<(), Diagnostic> {
      // Account for the fact this method was called
      let attrs = ATTRS.get_or_init(|| AttributeParseState::default());
      attrs.checks.fetch_add(1, Ordering::SeqCst);

      let mut errors = Vec::new();
      for (used, attr) in self.attrs.iter() {
        if used.get() {
            continue
        }
        let span = match attr {
          $(BindgenAttr::$variant(span, ..) => span,)*
        };
        errors.push(Diagnostic::span_error(*span, "unused #[napi] attribute"));
      }
      Diagnostic::from_vec(errors)
    }

    #[cfg(not(feature = "strict"))]
    #[allow(unused)]
    pub fn check_used(&self) -> Result<(), Diagnostic> {
        // Account for the fact this method was called
      let attrs = ATTRS.get_or_init(AttributeParseState::default);
      attrs.checks.fetch_add(1, Ordering::SeqCst);
      Ok(())
    }
  };

  (@method $name:ident, $variant:ident(Span, String, Span)) => {
    #[allow(unused)]
    pub fn $name(&self) -> Option<(&str, Span)> {
      self.attrs
        .iter()
        .filter_map(|a| match &a.1 {
          BindgenAttr::$variant(_, s, span) => {
            a.0.set(true);
            Some((&s[..], *span))
          }
          _ => None,
        })
        .next()
    }
  };

  (@method $name:ident, $variant:ident(Span, Option<(String, Span)>)) => {
    #[allow(unused)]
    pub fn $name(&self) -> Option<Option<&(String, Span)>> {
      self.attrs
        .iter()
        .filter_map(|a| match &a.1 {
          BindgenAttr::$variant(_, s) => {
            a.0.set(true);
            Some(s.as_ref())
          }
          _ => None,
        })
        .next()
    }
  };

  (@method $name:ident, $variant:ident(Span, Option<bool>), $default_value:literal) => {
    #[allow(unused)]
    pub fn $name(&self) -> bool {
      self.attrs
        .iter()
        .filter_map(|a| match &a.1 {
          BindgenAttr::$variant(_, s) => {
            a.0.set(true);
            *s
          }
          _ => None,
        })
        .next()
        .unwrap_or($default_value)
    }
  };

  (@method $name:ident, $variant:ident(Span, Vec<String>, Vec<Span>)) => {
    #[allow(unused)]
    pub fn $name(&self) -> Option<(&[String], &[Span])> {
      self.attrs
        .iter()
        .filter_map(|a| match &a.1 {
          BindgenAttr::$variant(_, ss, spans) => {
            a.0.set(true);
            Some((&ss[..], &spans[..]))
          }
          _ => None,
        })
        .next()
      }
  };

  (@method $name:ident, $variant:ident(Span, $($other:tt)*)) => {
    #[allow(unused)]
    pub fn $name(&self) -> Option<&$($other)*> {
      self.attrs
        .iter()
        .filter_map(|a| match &a.1 {
          BindgenAttr::$variant(_, s) => {
            a.0.set(true);
            Some(s)
          }
          _ => None,
        })
        .next()
      }
  };

  (@method $name:ident, $variant:ident($($other:tt)*)) => {
    #[allow(unused)]
    pub fn $name(&self) -> Option<&$($other)*> {
      self.attrs
        .iter()
        .filter_map(|a| match &a.1 {
          BindgenAttr::$variant(s) => {
            a.0.set(true);
            Some(s)
          }
          _ => None,
        })
        .next()
    }
  };
}

impl BindgenAttrs {
  /// Find and parse the napi attributes.
  pub fn find(attrs: &mut Vec<syn::Attribute>) -> Result<BindgenAttrs, Diagnostic> {
    for (index, attr) in attrs.iter().enumerate() {
      let attr = BindgenAttrs::try_from(attr)?;
      if attr.exists {
        attrs.remove(index);

        return Ok(attr);
      }
    }

    Ok(BindgenAttrs::default())
  }

  attrgen!(methods);
}

impl TryFrom<&Attribute> for BindgenAttrs {
  type Error = Diagnostic;

  fn try_from(attr: &Attribute) -> Result<Self, Self::Error> {
    let mut ret = BindgenAttrs {
      exists: false,
      attrs: vec![],
      span: Span::call_site(),
    };

    let is_napi =
      attr.path().segments.last().map(|s| s.ident.to_string()) == Some("napi".to_string());
    let is_cfg_attr = attr
      .meta
      .path()
      .segments
      .first()
      .map(|s| s.ident.to_string())
      == Some("cfg_attr".to_string());

    if is_napi {
      ret.exists = true;
      ret.span = attr.span();

      let tts = attr.meta.to_token_stream().into_iter();
      let group = match tts.last() {
        // #[napi(xxx)]
        //   ^^^^^^^^^
        Some(TokenTree::Group(d)) => d,
        // #[napi]
        //   ^^^^
        Some(TokenTree::Ident(_)) => parse_quote!(()),
        _ => bail_span!(attr, "invalid #[napi] attribute"),
      };

      if group.delimiter() != Delimiter::Parenthesis {
        bail_span!(attr, "malformed #[napi] attribute");
      }

      let mut attrs: BindgenAttrs = syn::parse2(group.stream())?;
      ret.attrs.append(&mut attrs.attrs);
    }

    if is_cfg_attr {
      let cfg_attr_list = attr.meta.require_list()?;
      // #[cfg_attr(condition, attr_to_apply)]
      // We parse the arguments of cfg_attr.
      let mut args_iter = cfg_attr_list
        .parse_args_with(syn::punctuated::Punctuated::<syn::Meta, Token![,]>::parse_terminated)?
        .into_iter();
      if let Some(arg) = args_iter.next_back() {
        if arg.path().segments.last().map(|s| s.ident.to_string()) == Some("napi".to_string()) {
          ret.exists = true;
          ret.span = arg.span();
          let tts = arg.to_token_stream().into_iter();
          let group = match tts.last() {
            // #[napi(xxx)]
            //   ^^^^^^^^^
            Some(TokenTree::Group(d)) => d,
            // #[napi]
            //   ^^^^
            Some(TokenTree::Ident(_)) => parse_quote!(()),
            _ => bail_span!(attr, "invalid #[napi] attribute"),
          };

          if group.delimiter() != Delimiter::Parenthesis {
            bail_span!(attr, "malformed #[napi] attribute");
          }

          let mut attrs: BindgenAttrs = syn::parse2(group.stream())?;
          ret.attrs.append(&mut attrs.attrs);
        }
      }
    }

    Ok(ret)
  }
}

impl Default for BindgenAttrs {
  fn default() -> BindgenAttrs {
    // Add 1 to the list of parsed attribute sets. We'll use this counter to
    // sanity check that we call `check_used` an appropriate number of
    // times.
    let attrs = ATTRS.get_or_init(AttributeParseState::default);
    attrs.parsed.fetch_add(1, Ordering::SeqCst);
    BindgenAttrs {
      span: Span::call_site(),
      attrs: Vec::new(),
      exists: false,
    }
  }
}

macro_rules! gen_bindgen_attr {
  ($( ($method:ident, $variant:ident($($associated_data:tt)*) $($extra_tokens:tt)*) ,)*) => {
    /// The possible attributes in the `#[napi]`.
    #[derive(Debug)]
    #[allow(unused)]
    pub enum BindgenAttr {
      $($variant($($associated_data)*)),*
    }
  }
}

attrgen!(gen_bindgen_attr);

pub fn record_struct(ident: &Ident, js_name: String, opts: &BindgenAttrs) {
  let state = STRUCTS.get_or_init(StructParseState::default);
  let mut map = state.parsed.lock().unwrap();
  let struct_name = ident.to_string();

  map.insert(
    struct_name,
    ParsedStruct {
      js_name,
      ctor_defined: opts.constructor().is_some(),
    },
  );
}

pub fn check_recorded_struct_for_impl(ident: &Ident, opts: &BindgenAttrs) -> BindgenResult<String> {
  let state = STRUCTS.get_or_init(StructParseState::default);
  let mut map = state.parsed.lock().unwrap();
  let struct_name = ident.to_string();
  if let Some(parsed) = map.get_mut(&struct_name) {
    if opts.constructor().is_some() && !cfg!(debug_assertions) {
      if parsed.ctor_defined {
        bail_span!(
          ident,
          "Constructor has already been defined for struct `{}`",
          &struct_name
        );
      } else {
        parsed.ctor_defined = true;
      }
    }

    Ok(parsed.js_name.clone())
  } else {
    bail_span!(
      ident,
      "Did not find struct `{}` parsed before expand #[napi] for impl",
      &struct_name,
    )
  }
}

impl Parse for BindgenAttrs {
  fn parse(input: ParseStream) -> syn::Result<Self> {
    let mut attrs = BindgenAttrs::default();
    if input.is_empty() {
      return Ok(attrs);
    }

    let opts = syn::punctuated::Punctuated::<_, syn::token::Comma>::parse_terminated(input)?;
    attrs.attrs = opts.into_iter().map(|c| (Cell::new(false), c)).collect();
    Ok(attrs)
  }
}

impl Parse for BindgenAttr {
  fn parse(input: ParseStream) -> syn::Result<Self> {
    let original = input.fork();
    let attr: AnyIdent = input.parse()?;
    let attr = attr.0;
    let attr_span = attr.span();
    let attr_string = attr.to_string();
    let raw_attr_string = format!("r#{attr_string}");

    macro_rules! parsers {
      ($(($name:ident, $($contents:tt)*),)*) => {
        $(
          if attr_string == stringify!($name) || raw_attr_string == stringify!($name) {
            parsers!(
              @parser
              $($contents)*
            );
          }
        )*
      };

      (@parser $variant:ident(Span)) => ({
        return Ok(BindgenAttr::$variant(attr_span));
      });

      (@parser $variant:ident(Span, Ident)) => ({
        input.parse::<Token![=]>()?;
        let ident = input.parse::<AnyIdent>()?.0;
        return Ok(BindgenAttr::$variant(attr_span, ident))
      });

      (@parser $variant:ident(Span, Option<Ident>)) => ({
        if input.parse::<Token![=]>().is_ok() {
          let ident = input.parse::<AnyIdent>()?.0;
          return Ok(BindgenAttr::$variant(attr_span, Some(ident)))
        } else {
          return Ok(BindgenAttr::$variant(attr_span, None));
        }
      });

        (@parser $variant:ident(Span, syn::Path)) => ({
            input.parse::<Token![=]>()?;
            return Ok(BindgenAttr::$variant(attr_span, input.parse()?));
        });

        (@parser $variant:ident(Span, syn::Expr)) => ({
            input.parse::<Token![=]>()?;
            return Ok(BindgenAttr::$variant(attr_span, input.parse()?));
        });

        (@parser $variant:ident(Span, String, Span)) => ({
          input.parse::<Token![=]>()?;
          let (val, span) = match input.parse::<syn::LitStr>() {
            Ok(str) => (str.value(), str.span()),
            Err(_) => {
              let ident = input.parse::<AnyIdent>()?.0;
              (ident.to_string(), ident.span())
            }
          };
          return Ok(BindgenAttr::$variant(attr_span, val, span))
        });

        (@parser $variant:ident(Span, Option<(String, Span)>)) => ({
          if let Ok(_) = input.parse::<Token![=]>() {
            let val = match input.parse::<syn::LitStr>() {
              Ok(str) => Some((str.value(), str.span())),
              Err(_) => {
                let ident = input.parse::<AnyIdent>()?.0;
                Some((ident.to_string(), ident.span()))
              }
            };
            return Ok(BindgenAttr::$variant(attr_span, val))
          } else {
            return Ok(BindgenAttr::$variant(attr_span, None))
          }
        });

        (@parser $variant:ident(Span, Option<bool>), $default_value:literal) => ({
          if let Ok(_) = input.parse::<Token![=]>() {
            let (val, _) = match input.parse::<syn::LitBool>() {
              Ok(str) => (str.value(), str.span()),
              Err(_) => {
                let ident = input.parse::<AnyIdent>()?.0;
                (true, ident.span())
              }
            };
            return Ok::<BindgenAttr, syn::Error>(BindgenAttr::$variant(attr_span, Some(val)))
          } else {
            return Ok(BindgenAttr::$variant(attr_span, Some($default_value)))
          }
        });

        (@parser $variant:ident(Span, Vec<String>, Vec<Span>)) => ({
          input.parse::<Token![=]>()?;
          let (vals, spans) = match input.parse::<syn::ExprArray>() {
            Ok(exprs) => {
              let mut vals = vec![];
              let mut spans = vec![];

              for expr in exprs.elems.iter() {
                if let syn::Expr::Lit(syn::ExprLit {
                  lit: syn::Lit::Str(ref str),
                  ..
                }) = expr {
                  vals.push(str.value());
                  spans.push(str.span());
                } else {
                  return Err(syn::Error::new(expr.span(), "expected string literals"));
                }
              }

              (vals, spans)
            },
            Err(_) => {
              let ident = input.parse::<AnyIdent>()?.0;
              (vec![ident.to_string()], vec![ident.span()])
            }
          };
          return Ok(BindgenAttr::$variant(attr_span, vals, spans))
        });
      }

    attrgen!(parsers);

    Err(original.error("unknown attribute"))
  }
}
