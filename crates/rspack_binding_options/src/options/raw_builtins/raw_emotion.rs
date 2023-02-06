use swc_emotion::EmotionOptions;

pub fn transform_emotion(emotion: Option<String>) -> anyhow::Result<Option<EmotionOptions>> {
  match emotion {
    Some(emotion) => Ok(Some(serde_json::from_str(&emotion)?)),
    None => Ok(None),
  }
}
