use either::Either;
use rspack_core::ResourceData;
use swc_core::ecma::visit::Fold;
use swc_emotion::EmotionOptions;

pub fn plugin_emotion(emotion: Option<&EmotionOptions>, resource_data: ResourceData) -> impl Fold {
  if let Some(emotion_options) = &emotion {
    Either::Left(swc_emotion::emotion(
      emotion_options.clone(),
      &resource_data.resource_path,
      cm.clone(),
      comments,
    ))
  } else {
    Either::Right(noop())
  }
}
