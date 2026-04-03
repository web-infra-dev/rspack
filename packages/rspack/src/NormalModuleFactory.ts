import binding from '@rspack/binding';

import * as liteTapable from '@rspack/lite-tapable';
import {
  type JsHookUsageTracker,
  trackHookMapUsage,
  trackHookUsage,
} from './HookUsageTracker';
import type { ResolveData, ResourceDataWithData } from './Module';
import type {
  ResolveOptionsWithDependencyType,
  ResolverFactory,
} from './ResolverFactory';

export type NormalModuleCreateData =
  binding.JsNormalModuleFactoryCreateModuleArgs & {
    settings: {};
  };

export class NormalModuleFactory {
  hooks: {
    // TODO: second param resolveData
    resolveForScheme: liteTapable.HookMap<
      liteTapable.AsyncSeriesBailHook<[ResourceDataWithData], true | void>
    >;
    beforeResolve: liteTapable.AsyncSeriesBailHook<[ResolveData], false | void>;
    factorize: liteTapable.AsyncSeriesBailHook<[ResolveData], void>;
    resolve: liteTapable.AsyncSeriesBailHook<[ResolveData], void>;
    afterResolve: liteTapable.AsyncSeriesBailHook<[ResolveData], false | void>;
    createModule: liteTapable.AsyncSeriesBailHook<
      [NormalModuleCreateData, {}],
      void
    >;
  };

  resolverFactory: ResolverFactory;

  constructor(
    resolverFactory: ResolverFactory,
    hookUsageTracker?: JsHookUsageTracker,
  ) {
    this.hooks = {
      resolveForScheme: new liteTapable.HookMap(
        () => new liteTapable.AsyncSeriesBailHook(['resourceData']),
      ),
      beforeResolve: new liteTapable.AsyncSeriesBailHook(['resolveData']),
      factorize: new liteTapable.AsyncSeriesBailHook(['resolveData']),
      resolve: new liteTapable.AsyncSeriesBailHook(['resolveData']),
      afterResolve: new liteTapable.AsyncSeriesBailHook(['resolveData']),
      createModule: new liteTapable.AsyncSeriesBailHook([
        'createData',
        'resolveData',
      ]),
    };

    if (hookUsageTracker) {
      trackHookMapUsage(
        this.hooks.resolveForScheme,
        hookUsageTracker,
        binding.RegisterJsTapKind.NormalModuleFactoryResolveForScheme,
      );
      trackHookUsage(
        this.hooks.beforeResolve,
        hookUsageTracker,
        binding.RegisterJsTapKind.NormalModuleFactoryBeforeResolve,
      );
      trackHookUsage(
        this.hooks.factorize,
        hookUsageTracker,
        binding.RegisterJsTapKind.NormalModuleFactoryFactorize,
      );
      trackHookUsage(
        this.hooks.resolve,
        hookUsageTracker,
        binding.RegisterJsTapKind.NormalModuleFactoryResolve,
      );
      trackHookUsage(
        this.hooks.afterResolve,
        hookUsageTracker,
        binding.RegisterJsTapKind.NormalModuleFactoryAfterResolve,
      );
      trackHookUsage(
        this.hooks.createModule,
        hookUsageTracker,
        binding.RegisterJsTapKind.NormalModuleFactoryCreateModule,
      );
    }

    this.resolverFactory = resolverFactory;
  }

  getResolver(type: string, resolveOptions: ResolveOptionsWithDependencyType) {
    return this.resolverFactory.get(type, resolveOptions);
  }
}
