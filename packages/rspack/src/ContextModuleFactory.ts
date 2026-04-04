import binding from '@rspack/binding';
import * as liteTapable from '@rspack/lite-tapable';
import {
  BindingAsyncSeriesWaterfallHook,
  type HookUsageTracker,
} from './HookUsageTracker';
import type {
  ContextModuleFactoryAfterResolveResult,
  ContextModuleFactoryBeforeResolveResult,
} from './Module';

export class ContextModuleFactory {
  hooks: {
    beforeResolve: liteTapable.AsyncSeriesWaterfallHook<
      [ContextModuleFactoryBeforeResolveResult],
      ContextModuleFactoryBeforeResolveResult | void
    >;
    afterResolve: liteTapable.AsyncSeriesWaterfallHook<
      [ContextModuleFactoryAfterResolveResult],
      ContextModuleFactoryAfterResolveResult | void
    >;
  };
  constructor(hookUsageTracker?: HookUsageTracker) {
    this.hooks = {
      beforeResolve: hookUsageTracker
        ? new BindingAsyncSeriesWaterfallHook(
            ['resolveData'],
            hookUsageTracker,
            binding.CompilationHooks.ContextModuleFactoryBeforeResolve,
          )
        : new liteTapable.AsyncSeriesWaterfallHook(['resolveData']),
      afterResolve: hookUsageTracker
        ? new BindingAsyncSeriesWaterfallHook(
            ['resolveData'],
            hookUsageTracker,
            binding.CompilationHooks.ContextModuleFactoryAfterResolve,
          )
        : new liteTapable.AsyncSeriesWaterfallHook(['resolveData']),
    };
  }
}
