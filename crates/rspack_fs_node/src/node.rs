use napi::{Env, JsFunction, NapiRaw, Ref};

pub(crate) struct JsFunctionRef {
  env: Env,
  reference: Ref<()>,
}

impl JsFunctionRef {
  fn new(env: Env, f: JsFunction) -> napi::Result<Self> {
    Ok(Self {
      env,
      reference: env.create_reference(f)?,
    })
  }

  pub(crate) fn get(&self) -> napi::Result<JsFunction> {
    self.env.get_reference_value(&self.reference)
  }
}

impl Drop for JsFunctionRef {
  fn drop(&mut self) {
    let result = self.reference.unref(self.env);
    debug_assert!(result.is_ok());
  }
}

#[napi(object)]
pub struct NodeFS {
  pub write_file: JsFunction,
  pub mkdir: JsFunction,
  pub mkdirp: JsFunction,
}

pub(crate) trait TryIntoNodeFSRef {
  fn try_into_node_fs_ref(self, env: &Env) -> napi::Result<NodeFSRef>;
}

impl TryIntoNodeFSRef for NodeFS {
  fn try_into_node_fs_ref(self, env: &Env) -> napi::Result<NodeFSRef> {
    Ok(NodeFSRef {
      write_file: JsFunctionRef::new(*env, self.write_file)?,
      mkdir: JsFunctionRef::new(*env, self.mkdir)?,
      mkdirp: JsFunctionRef::new(*env, self.mkdirp)?,
    })
  }
}

pub(crate) struct NodeFSRef {
  pub(crate) write_file: JsFunctionRef,
  pub(crate) mkdir: JsFunctionRef,
  pub(crate) mkdirp: JsFunctionRef,
}

cfg_async! {
  use napi::{
    bindgen_prelude::FromNapiValue,
    JsUnknown,
    Either,
  };
  use napi_derive::napi;
  use rspack_napi_shared::threadsafe_function::ThreadsafeFunction;

  #[napi(object)]
  pub struct ThreadsafeNodeFS {
    pub write_file: JsFunction,
    pub mkdir: JsFunction,
    pub mkdirp: JsFunction,
  }

  trait TryIntoJsUnknown {
    fn try_into_js_unknown(self, env: &Env) -> napi::Result<JsUnknown>;
  }

  // Implement a few common types to avoid conflict with upperstream crates

  impl TryIntoJsUnknown for String {
    fn try_into_js_unknown(self, env: &Env) -> napi::Result<JsUnknown> {
      env.create_string_from_std(self).map(|v| v.into_unknown())
    }
  }

  impl TryIntoJsUnknown for Vec<u8> {
    fn try_into_js_unknown(self, env: &Env) -> napi::Result<JsUnknown> {
      env.create_buffer_with_data(self).map(|v| v.into_unknown())
    }
  }

  trait JsValuesTupleIntoVec {
    fn into_vec(self, env: &Env) -> napi::Result<Vec<JsUnknown>>;
  }

  impl<T: TryIntoJsUnknown> JsValuesTupleIntoVec for T {
    fn into_vec(self, env: &Env) -> napi::Result<Vec<JsUnknown>> {
      Ok(vec![<T as TryIntoJsUnknown>::try_into_js_unknown(self, env)?])
    }
  }

  macro_rules! impl_js_value_tuple_to_vec {
    ($($ident:ident),*) => {
      impl<$($ident: TryIntoJsUnknown),*> JsValuesTupleIntoVec for ($($ident,)*) {
        fn into_vec(self, env: &Env) -> napi::Result<Vec<JsUnknown>> {
          #[allow(non_snake_case)]
          let ($($ident,)*) = self;
          Ok(vec![$(<$ident as TryIntoJsUnknown>::try_into_js_unknown($ident, env)?),*])
        }
      }
    };
  }

  impl_js_value_tuple_to_vec!(A);
  impl_js_value_tuple_to_vec!(A, B);
  impl_js_value_tuple_to_vec!(A, B, C);

  pub(crate) trait TryIntoThreadsafeFunctionRef {
    fn try_into_tsfn_ref(self, env: &Env) -> napi::Result<ThreadsafeFunctionRef>;
  }

  pub(crate) trait TryIntoThreadsafeFunction<T, R> {
    fn try_into_tsfn(self, env: &Env) -> napi::Result<ThreadsafeFunction<T, R>>;
  }

  impl<T: JsValuesTupleIntoVec, R: FromNapiValue + Send + 'static> TryIntoThreadsafeFunction<T, R>
    for JsFunction
  {
    fn try_into_tsfn(self, env: &Env) -> napi::Result<ThreadsafeFunction<T, R>> {
      let mut tsfn: ThreadsafeFunction<T, R> =
        ThreadsafeFunction::create(env.raw(), unsafe { self.raw() }, 0, |ctx| {
          let (ctx, resolver) = ctx.split_into_parts();

          let env = ctx.env;
          let cb = ctx.callback;
          let result = <T as JsValuesTupleIntoVec>::into_vec(ctx.value, &env)?;
          let result = cb.call(None, &result);

          resolver.resolve::<R>(result, |_, v| Ok(v))
        })?;

      tsfn.unref(env)?;

      Ok(tsfn)
    }
  }

  impl TryIntoThreadsafeFunctionRef for ThreadsafeNodeFS {
    fn try_into_tsfn_ref(self, env: &Env) -> napi::Result<ThreadsafeFunctionRef> {
      Ok(ThreadsafeFunctionRef {
        write_file: self.write_file.try_into_tsfn(env)?,
        mkdir: self.mkdir.try_into_tsfn(env)?,
        mkdirp: self.mkdirp.try_into_tsfn(env)?,
      })
    }
  }

  pub(crate) struct ThreadsafeFunctionRef {
    pub(crate) write_file: ThreadsafeFunction<(String, Vec<u8>), ()>,
    pub(crate) mkdir: ThreadsafeFunction<String, ()>,
    pub(crate) mkdirp: ThreadsafeFunction<String, Either<String, ()>>,
  }
}
