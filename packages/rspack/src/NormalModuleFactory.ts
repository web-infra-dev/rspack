import binding from '@rspack/binding';

import * as liteTapable from '@rspack/lite-tapable';
import {
  BindingAsyncSeriesBailHook,
  type HookSubscriptionBitset,
} from './BindingHooks';
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
      BindingAsyncSeriesBailHook<[ResourceDataWithData], true | void>
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
    hookSubscriptionBitset: HookSubscriptionBitset,
  ) {
    this.hooks = {
      resolveForScheme: new liteTapable.HookMap(
        () =>
          new BindingAsyncSeriesBailHook(
            ['resourceData'],
            hookSubscriptionBitset,
            binding.CompilationHooks.NormalModuleFactoryResolveForScheme,
          ),
      ),
      beforeResolve: new BindingAsyncSeriesBailHook(
        ['resolveData'],
        hookSubscriptionBitset,
        binding.CompilationHooks.NormalModuleFactoryBeforeResolve,
      ),
      factorize: new BindingAsyncSeriesBailHook(
        ['resolveData'],
        hookSubscriptionBitset,
        binding.CompilationHooks.NormalModuleFactoryFactorize,
      ),
      resolve: new BindingAsyncSeriesBailHook(
        ['resolveData'],
        hookSubscriptionBitset,
        binding.CompilationHooks.NormalModuleFactoryResolve,
      ),
      afterResolve: new BindingAsyncSeriesBailHook(
        ['resolveData'],
        hookSubscriptionBitset,
        binding.CompilationHooks.NormalModuleFactoryAfterResolve,
      ),
      createModule: new BindingAsyncSeriesBailHook(
        ['createData', 'resolveData'],
        hookSubscriptionBitset,
        binding.CompilationHooks.NormalModuleFactoryCreateModule,
      ),
    };

    this.resolverFactory = resolverFactory;
  }

  getResolver(type: string, resolveOptions: ResolveOptionsWithDependencyType) {
    return this.resolverFactory.get(type, resolveOptions);
  }
}
