import type binding from '@rspack/binding';

import * as liteTapable from '@rspack/lite-tapable';
import type { ResolveData, ResourceDataWithData } from './Module';
import type {
  ResolveOptionsWithDependencyType,
  ResolverFactory,
} from './ResolverFactory';

export type NormalModuleCreateData =
  binding.JsNormalModuleFactoryCreateModuleArgs & {
    settings: {};
  };

export function createResolveData(data: binding.JsResolveData): ResolveData {
  // TODO(v3): Remove resolveData.fileDependencies, resolveData.contextDependencies,
  // and resolveData.missingDependencies.
  // These arrays are kept only for compatibility and initialized in JavaScript
  // to avoid transferring unused dependencies.
  const resolveData = data as ResolveData;
  resolveData.fileDependencies = [];
  resolveData.contextDependencies = [];
  resolveData.missingDependencies = [];
  return resolveData;
}

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

  constructor(resolverFactory: ResolverFactory) {
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
    this.resolverFactory = resolverFactory;
  }

  getResolver(type: string, resolveOptions: ResolveOptionsWithDependencyType) {
    return this.resolverFactory.get(type, resolveOptions);
  }
}
