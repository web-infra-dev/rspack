import binding from '@rspack/binding';
import * as liteTapable from '@rspack/lite-tapable';
import { type JsHookUsageTracker, trackHookUsage } from './HookUsageTracker';
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
  constructor(hookUsageTracker?: JsHookUsageTracker) {
    this.hooks = {
      beforeResolve: new liteTapable.AsyncSeriesWaterfallHook(['resolveData']),
      afterResolve: new liteTapable.AsyncSeriesWaterfallHook(['resolveData']),
    };

    if (hookUsageTracker) {
      trackHookUsage(
        this.hooks.beforeResolve,
        hookUsageTracker,
        binding.RegisterJsTapKind.ContextModuleFactoryBeforeResolve,
      );
      trackHookUsage(
        this.hooks.afterResolve,
        hookUsageTracker,
        binding.RegisterJsTapKind.ContextModuleFactoryAfterResolve,
      );
    }
  }
}
