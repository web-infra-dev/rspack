use napi::Status;
use swc_emotion::EmotionOptions;

pub fn transform_emotion(emotion: Option<String>) -> anyhow::Result<Option<EmotionOptions>> {
  match emotion {
    Some(emotion) => Ok(Some(serde_json::from_str(&emotion).map_err(|e| {
      napi::Error::new(
        Status::InvalidArg,
        format!("Failed to resolve configuration `builtins.emotion`:\n{e}"),
      )
    })?)),
    None => Ok(None),
  }
}
