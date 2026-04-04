import type {
  JsAfterEmitData,
  JsAfterTemplateExecutionData,
  JsAlterAssetTagGroupsData,
  JsAlterAssetTagsData,
  JsBeforeAssetTagGenerationData,
  JsBeforeEmitData,
} from '@rspack/binding';
import binding from '@rspack/binding';
import * as liteTapable from '@rspack/lite-tapable';
import { type Compilation, checkCompilation } from '../../Compilation';
import {
  BindingAsyncSeriesWaterfallHook,
  COMPILATION_HOOK_SUBSCRIPTION_BITSETS,
} from '../../BindingHooks';
import type { HtmlRspackPluginOptions } from './options';

const compilationHooksMap: WeakMap<Compilation, HtmlRspackPluginHooks> =
  new WeakMap();

type ExtraPluginHookData = {
  plugin: {
    options: HtmlRspackPluginOptions;
  };
};

export type HtmlRspackPluginHooks = {
  beforeAssetTagGeneration: liteTapable.AsyncSeriesWaterfallHook<
    [JsBeforeAssetTagGenerationData & ExtraPluginHookData]
  >;
  alterAssetTags: liteTapable.AsyncSeriesWaterfallHook<
    [JsAlterAssetTagsData & ExtraPluginHookData]
  >;
  alterAssetTagGroups: liteTapable.AsyncSeriesWaterfallHook<
    [JsAlterAssetTagGroupsData & ExtraPluginHookData]
  >;
  afterTemplateExecution: liteTapable.AsyncSeriesWaterfallHook<
    [JsAfterTemplateExecutionData & ExtraPluginHookData]
  >;
  beforeEmit: liteTapable.AsyncSeriesWaterfallHook<
    [JsBeforeEmitData & ExtraPluginHookData]
  >;
  afterEmit: liteTapable.AsyncSeriesWaterfallHook<
    [JsAfterEmitData & ExtraPluginHookData]
  >;
};

export const getPluginHooks = (compilation: Compilation) => {
  checkCompilation(compilation);

  let hooks = compilationHooksMap.get(compilation);
  if (hooks === undefined) {
    const hookSubscriptionBitset = COMPILATION_HOOK_SUBSCRIPTION_BITSETS.get(
      compilation.compiler,
    )!;
    hooks = {
      beforeAssetTagGeneration: new BindingAsyncSeriesWaterfallHook(
        ['data'],
        hookSubscriptionBitset,
        binding.CompilationHooks.HtmlPluginBeforeAssetTagGeneration,
      ),
      alterAssetTags: new BindingAsyncSeriesWaterfallHook(
        ['data'],
        hookSubscriptionBitset,
        binding.CompilationHooks.HtmlPluginAlterAssetTags,
      ),
      alterAssetTagGroups: new BindingAsyncSeriesWaterfallHook(
        ['data'],
        hookSubscriptionBitset,
        binding.CompilationHooks.HtmlPluginAlterAssetTagGroups,
      ),
      afterTemplateExecution: new BindingAsyncSeriesWaterfallHook(
        ['data'],
        hookSubscriptionBitset,
        binding.CompilationHooks.HtmlPluginAfterTemplateExecution,
      ),
      beforeEmit: new BindingAsyncSeriesWaterfallHook(
        ['data'],
        hookSubscriptionBitset,
        binding.CompilationHooks.HtmlPluginBeforeEmit,
      ),
      afterEmit: new BindingAsyncSeriesWaterfallHook(
        ['data'],
        hookSubscriptionBitset,
        binding.CompilationHooks.HtmlPluginAfterEmit,
      ),
    };
    compilationHooksMap.set(compilation, hooks);
  }
  return hooks;
};

export const cleanPluginHooks = (compilation: Compilation) => {
  compilationHooksMap.delete(compilation);
};
