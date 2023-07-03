use crate::JsParserHookDriver;

pub struct JsParserPluginContext<'me> {
  pub parser: &'me mut JsParserHookDriver,
}

pub trait JsParserPlugin {
  fn apply(&mut self, ctx: &mut JsParserPluginContext);
}
