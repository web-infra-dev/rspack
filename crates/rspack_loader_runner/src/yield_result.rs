use std::ops::Range;

use rspack_error::Result;
use rspack_sources::SourceMap;

use crate::{
  AdditionalData, Content, LoaderContext, LoaderDependencies, LoaderYieldInput, ParseMeta,
};

pub struct LoaderItemUpdate {
  pub normal_executed: bool,
  pub pitch_executed: bool,
  pub filter_matched: bool,
  pub data: serde_json::Value,
}

pub struct LoaderOutput {
  pub content: Option<Content>,
  pub source_map: Option<SourceMap<'static>>,
  pub additional_data: Option<AdditionalData>,
  pub loader_items: Vec<LoaderItemUpdate>,
  pub loader_index: i32,
  pub parse_meta: ParseMeta,
}

pub struct LoaderExecutionResult {
  pub cacheable: bool,
  pub dependencies: LoaderDependencies,
  pub added_dependencies: LoaderDependencies,
  pub removed_dependencies: LoaderDependencies,
  pub output: Result<LoaderOutput>,
}

pub enum LoaderYieldResult {
  Unhandled(LoaderYieldInput),
  SkipPitch {
    input: LoaderYieldInput,
    range: Range<usize>,
  },
  Executed(LoaderExecutionResult),
}

impl LoaderYieldResult {
  pub fn merge<Context: Send>(self, context: &mut LoaderContext<Context>) -> Result<()> {
    match self {
      Self::Unhandled(input) => input.restore(context),
      Self::SkipPitch { input, range } => {
        context.loader_index = range.end as i32;
        for index in range {
          context.loader_items[index].set_pitch_executed();
        }
        input.restore(context);
      }
      Self::Executed(result) => {
        context.cacheable = result.cacheable;
        context.replace_dependencies(
          result.dependencies,
          result.added_dependencies,
          result.removed_dependencies,
        );
        let output = result.output?;
        context.__finish_with((output.content, output.source_map, output.additional_data));
        context.loader_items = context
          .loader_items
          .drain(..)
          .zip(output.loader_items)
          .map(|(mut item, update)| {
            if update.normal_executed {
              item.set_normal_executed();
            }
            if update.pitch_executed {
              item.set_pitch_executed();
            }
            if update.filter_matched {
              item.set_filter_matched();
            }
            item.set_data(update.data);
            item.set_finish_called();
            item
          })
          .collect();
        context.loader_index = output.loader_index;
        context.parse_meta.extend(output.parse_meta);
      }
    }
    Ok(())
  }
}
