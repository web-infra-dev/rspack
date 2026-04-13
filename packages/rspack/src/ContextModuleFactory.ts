import binding from '@rspack/binding';
import * as liteTapable from '@rspack/lite-tapable';
import {
  BindingAsyncSeriesWaterfallHook,
  type HookSubscriptionBitset,
} from './BindingHooks';
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

  constructor(hookSubscriptionBitset: HookSubscriptionBitset) {
    this.hooks = {
      beforeResolve: new BindingAsyncSeriesWaterfallHook(
        ['resolveData'],
        hookSubscriptionBitset,
        binding.CompilationHooks.ContextModuleFactoryBeforeResolve,
      ),
      afterResolve: new BindingAsyncSeriesWaterfallHook(
        ['resolveData'],
        hookSubscriptionBitset,
        binding.CompilationHooks.ContextModuleFactoryAfterResolve,
      ),
    };
  }
}
