import binding from '@rspack/binding';
import * as liteTapable from '@rspack/lite-tapable';
import { type Compilation, getOrCreateCompilationHooks } from './Compilation';
import type { LoaderContext } from './config';
import { Module } from './Module';
import { COMPILATION_HOOKS_MAP_SYMBOL } from './util/symbols';

export interface NormalModuleCompilationHooks {
  loader: liteTapable.SyncHook<[LoaderContext, Module]>;
  readResource: liteTapable.HookMap<
    liteTapable.AsyncSeriesBailHook<[LoaderContext], string | Buffer>
  >;
}

export class NormalModule extends Module {
  static readonly #properties: PropertyDescriptorMap = {
    resource: {
      enumerable: true,
      configurable: true,
      writable: true,
      value: undefined,
    },
    request: {
      enumerable: true,
      configurable: true,
      writable: true,
      value: undefined,
    },
    userRequest: {
      enumerable: true,
      configurable: true,
      writable: true,
      value: undefined,
    },
    rawRequest: {
      enumerable: true,
      configurable: true,
      writable: true,
      value: undefined,
    },
    resourceResolveData: {
      enumerable: true,
      configurable: true,
      writable: true,
      value: undefined,
    },
    loaders: {
      enumerable: true,
      configurable: true,
      writable: true,
      value: undefined,
    },
    matchResource: {
      enumerable: true,
      configurable: true,
      get(this: NormalModule): string | undefined {
        return binding.normalModuleGetMatchResource(this);
      },
      set(this: NormalModule, value: string | undefined) {
        binding.normalModuleSetMatchResource(this, value);
      },
    },
    error: {
      enumerable: true,
      configurable: true,
      get(this: NormalModule): binding.RspackError | undefined {
        return binding.normalModuleGetError(this);
      },
    },
    ...Module._properties,
  };

  declare readonly resource: string;
  declare readonly request: string;
  declare readonly userRequest: string;
  declare readonly rawRequest: string;
  declare readonly resourceResolveData:
    Readonly<binding.JsResourceData> | undefined;
  declare readonly loaders: binding.JsLoaderItem[];
  declare matchResource: string | undefined;
  declare readonly error: binding.RspackError | undefined;

  constructor(data?: binding.JsModuleData) {
    super(data, NormalModule.#properties);
    if (data?.normal) {
      this.resource = data.normal.resource;
      this.request = data.normal.request;
      this.userRequest = data.normal.userRequest;
      this.rawRequest = data.normal.rawRequest;
      this.resourceResolveData = data.normal.resourceResolveData;
      this.loaders = data.normal.loaders;
    }
  }

  static getCompilationHooks(
    compilation: Compilation,
  ): NormalModuleCompilationHooks {
    if (!(COMPILATION_HOOKS_MAP_SYMBOL in compilation)) {
      throw new TypeError(
        "The 'compilation' argument must be an instance of Compilation",
      );
    }

    return getOrCreateCompilationHooks(compilation, compilation, () => ({
      loader: new liteTapable.SyncHook(['loaderContext', 'module']),
      readResource: new liteTapable.HookMap(
        () => new liteTapable.AsyncSeriesBailHook(['loaderContext']),
      ),
    }));
  }
}
