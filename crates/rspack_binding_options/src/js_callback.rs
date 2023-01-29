use std::marker::PhantomData;

use napi::{
  bindgen_prelude::*,
  threadsafe_function::{ErrorStrategy, ThreadSafeCallContext, ThreadsafeFunction},
  JsUnknown,
};

pub trait JsValueTupleIntoJsUnknownVec {
  fn to_vec(self, env: &Env) -> Result<Vec<JsUnknown>>;
}

macro_rules! impl_tuple_to_vec {
  ( $( $parameter:ident ),+ $( , )* ) => {
    #[allow(unused_parens)]
    impl< $( $parameter ),+ > JsValueTupleIntoJsUnknownVec for ( $( $parameter ),+, )
    where $( $parameter: ToNapiValue ),+
    {
      #[allow(non_snake_case)]
      fn to_vec(self, env: &Env) -> Result<Vec<JsUnknown>> {
        let ( $( $parameter ),+, ) = self;
        let vec = unsafe {
          vec![
            $(
              JsUnknown::from_napi_value(
                env.raw(),
                ToNapiValue::to_napi_value(env.raw(), $parameter)?,
              )?
            ),+
          ]
        };
        Ok(vec)
      }
    }
  };
}

impl_tuple_to_vec!(A);
impl_tuple_to_vec!(A, B);
impl_tuple_to_vec!(A, B, C);
impl_tuple_to_vec!(A, B, C, D);
impl_tuple_to_vec!(A, B, C, D, E);
impl_tuple_to_vec!(A, B, C, D, E, F);
impl_tuple_to_vec!(A, B, C, D, E, F, G);
impl_tuple_to_vec!(A, B, C, D, E, F, G, H);
impl_tuple_to_vec!(A, B, C, D, E, F, G, H, I);
impl_tuple_to_vec!(A, B, C, D, E, F, G, H, I, J);

trait JsCallbackArgs: JsValueTupleIntoJsUnknownVec + Send + Sync + 'static {}
impl<T: JsValueTupleIntoJsUnknownVec + Send + Sync + 'static> JsCallbackArgs for T {}
trait JsCallbackRet: FromNapiValue + ValidateNapiValue + Send + 'static {}
impl<T: FromNapiValue + ValidateNapiValue + Send + 'static> JsCallbackRet for T {}

struct JsCallback<Args: JsCallbackArgs, Ret: JsCallbackRet> {
  _args: PhantomData<Args>,
  _ret: PhantomData<Ret>,
  ts_fn: ThreadsafeFunction<Args, ErrorStrategy::Fatal>,
}

impl<Args: JsCallbackArgs, Ret: JsCallbackRet> JsCallback<Args, Ret> {
  pub fn new(js_fn: JsFunction) -> napi::Result<Self> {
    let ts_fn: ThreadsafeFunction<Args, ErrorStrategy::Fatal> = js_fn
      .create_threadsafe_function(0, |ctx: ThreadSafeCallContext<Args>| {
        ctx.value.to_vec(&ctx.env)
      })?;
    Ok(Self {
      _args: PhantomData,
      _ret: PhantomData,
      ts_fn,
    })
  }

  /// This method is already handle case return Promise<Ret>
  pub(crate) async fn call_async(&self, args: Args) -> napi::Result<Ret> {
    let ret: Either<Ret, Promise<Ret>> = self.ts_fn.call_async(args).await?;

    match ret {
      Either::A(ret) => Ok(ret),
      Either::B(promise) => promise.await,
    }
  }
}

impl<Args: JsCallbackArgs, Ret: JsCallbackRet> Clone for JsCallback<Args, Ret> {
  fn clone(&self) -> Self {
    Self {
      _args: PhantomData,
      _ret: PhantomData,
      ts_fn: self.ts_fn.clone(),
    }
  }
}

#[allow(unused)]
async fn example() {
  // To call the javascript function having signature `fn(a: number, b: string): string | Promise<string>`
  let js_fn: JsFunction = todo!();
  let js_callback = JsCallback::<(f64, String), String>::new(js_fn).unwrap();

  // Call from rust side

  let ret = js_callback
    .call_async((1.0, "hello".to_string()))
    .await
    .unwrap();

  // To call the javascript function having signature `fn(a: number): string | Promise<string>`
  let js_fn: JsFunction = todo!();
  let js_callback = JsCallback::<(f64,), String>::new(js_fn).unwrap();

  // Call from rust side

  let ret = js_callback.call_async((1.0,)).await.unwrap();
}
