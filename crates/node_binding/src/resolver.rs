use std::sync::Arc;
use std::{cell::RefCell, path::Path};

use napi::{bindgen_prelude::ToNapiValue, Either};
use napi_derive::napi;
use rspack_core::{ResolveOptionsWithDependencyType, Resolver, ResolverFactory};
use rspack_napi::OneShotInstanceRef;
use rustc_hash::FxHashMap as HashMap;

use crate::raw_resolve::{
  normalize_raw_resolve_options_with_dependency_type, RawResolveOptionsWithDependencyType,
};

#[napi]
#[derive(Debug)]
pub struct JsResolver {
  resolver_factory: Arc<ResolverFactory>,
  resolver: Arc<Resolver>,
  options: ResolveOptionsWithDependencyType,
}

#[napi]
impl JsResolver {
  #[napi(ts_return_type = "string | false")]
  pub fn resolve_sync(&self, path: String, request: String) -> napi::Result<Either<String, bool>> {
    match self.resolver.resolve(Path::new(&path), &request) {
      Ok(rspack_core::ResolveResult::Resource(resource)) => Ok(Either::A(resource.full_path())),
      Ok(rspack_core::ResolveResult::Ignored) => Ok(Either::B(false)),
      Err(err) => Err(napi::Error::from_reason(format!("{:?}", err))),
    }
  }

  #[napi(ts_args_type = "JsResolver")]
  pub fn with_options(
    &self,
    raw: Option<RawResolveOptionsWithDependencyType>,
  ) -> napi::Result<JsResolverWrapper> {
    let options =
      normalize_raw_resolve_options_with_dependency_type(raw, self.options.resolve_to_context);
    match options {
      Ok(mut options) => {
        options.resolve_options = match options.resolve_options.take() {
          Some(resolve_options) => match &self.options.resolve_options {
            Some(origin_resolve_options) => Some(Box::new(
              resolve_options.merge(*origin_resolve_options.clone()),
            )),
            None => Some(resolve_options),
          },
          None => self.options.resolve_options.clone(),
        };

        Ok(JsResolverWrapper::new(
          self.resolver_factory.clone(),
          options,
        ))
      }
      Err(e) => Err(napi::Error::from_reason(format!("{e}"))),
    }
  }
}

type ResolverInstanceRefs = RefCell<HashMap<JsResolverWrapper, OneShotInstanceRef<JsResolver>>>;

thread_local! {
  static RESOLVER_INSTANCE_REFS: ResolverInstanceRefs = Default::default();
}

#[derive(Clone)]
pub struct JsResolverWrapper {
  resolver_factory: Arc<ResolverFactory>,
  resolver: Arc<Resolver>,
  options: ResolveOptionsWithDependencyType,
}

impl JsResolverWrapper {
  pub fn new(
    resolver_factory: Arc<ResolverFactory>,
    options: ResolveOptionsWithDependencyType,
  ) -> Self {
    let resolver = resolver_factory.get(options.clone());
    Self {
      resolver_factory,
      resolver,
      options,
    }
  }
}

impl PartialEq for JsResolverWrapper {
  fn eq(&self, other: &Self) -> bool {
    Arc::ptr_eq(&self.resolver_factory, &other.resolver_factory) && self.options == other.options
  }
}

impl Eq for JsResolverWrapper {}

impl std::hash::Hash for JsResolverWrapper {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    Arc::as_ptr(&self.resolver_factory).hash(state);
    self.options.hash(state);
  }
}

impl ToNapiValue for JsResolverWrapper {
  unsafe fn to_napi_value(
    env: napi::sys::napi_env,
    val: Self,
  ) -> napi::Result<napi::sys::napi_value> {
    RESOLVER_INSTANCE_REFS.with(|ref_cell| {
      let mut refs = ref_cell.borrow_mut();
      match refs.entry(val.clone()) {
        std::collections::hash_map::Entry::Occupied(entry) => {
          ToNapiValue::to_napi_value(env, entry.get())
        }
        std::collections::hash_map::Entry::Vacant(entry) => {
          let js_resolver = JsResolver {
            resolver_factory: val.resolver_factory,
            resolver: val.resolver,
            options: val.options,
          };
          let r = entry.insert(OneShotInstanceRef::new(env, js_resolver)?);
          ToNapiValue::to_napi_value(env, r)
        }
      }
    })
  }
}
