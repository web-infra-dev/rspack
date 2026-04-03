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
  COMPILER_HOOK_USAGE_TRACKERS,
  trackHookUsage,
} from '../../HookUsageTracker';
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
    hooks = {
      beforeAssetTagGeneration: new liteTapable.AsyncSeriesWaterfallHook([
        'data',
      ]),
      alterAssetTags: new liteTapable.AsyncSeriesWaterfallHook(['data']),
      alterAssetTagGroups: new liteTapable.AsyncSeriesWaterfallHook(['data']),
      afterTemplateExecution: new liteTapable.AsyncSeriesWaterfallHook([
        'data',
      ]),
      beforeEmit: new liteTapable.AsyncSeriesWaterfallHook(['data']),
      afterEmit: new liteTapable.AsyncSeriesWaterfallHook(['data']),
    };
    const hookUsageTracker = COMPILER_HOOK_USAGE_TRACKERS.get(
      compilation.compiler,
    )!;
    trackHookUsage(
      hooks.beforeAssetTagGeneration,
      hookUsageTracker,
      binding.RegisterJsTapKind.HtmlPluginBeforeAssetTagGeneration,
    );
    trackHookUsage(
      hooks.alterAssetTags,
      hookUsageTracker,
      binding.RegisterJsTapKind.HtmlPluginAlterAssetTags,
    );
    trackHookUsage(
      hooks.alterAssetTagGroups,
      hookUsageTracker,
      binding.RegisterJsTapKind.HtmlPluginAlterAssetTagGroups,
    );
    trackHookUsage(
      hooks.afterTemplateExecution,
      hookUsageTracker,
      binding.RegisterJsTapKind.HtmlPluginAfterTemplateExecution,
    );
    trackHookUsage(
      hooks.beforeEmit,
      hookUsageTracker,
      binding.RegisterJsTapKind.HtmlPluginBeforeEmit,
    );
    trackHookUsage(
      hooks.afterEmit,
      hookUsageTracker,
      binding.RegisterJsTapKind.HtmlPluginAfterEmit,
    );
    compilationHooksMap.set(compilation, hooks);
  }
  return hooks;
};

export const cleanPluginHooks = (compilation: Compilation) => {
  compilationHooksMap.delete(compilation);
};
