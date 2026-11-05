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
  #resource: string | null = null;
  #request: string | null = null;
  #userRequest: string | null = null;
  #rawRequest: string | null = null;
  #resourceResolveData: Readonly<binding.JsResourceData> | undefined | null =
    null;
  #loaders: binding.JsLoaderItem[] | null = null;

  static readonly #properties: PropertyDescriptorMap = {
    resource: {
      enumerable: true,
      configurable: true,
      get(this: NormalModule): string {
        if (this.#resource === null) {
          this.#resource = binding.normalModuleGetResource(this);
        }
        return this.#resource;
      },
      set(this: NormalModule, value: string) {
        this.#resource = value;
      },
    },
    request: {
      enumerable: true,
      configurable: true,
      get(this: NormalModule): string {
        if (this.#request === null) {
          this.#request = binding.normalModuleGetRequest(this);
        }
        return this.#request;
      },
      set(this: NormalModule, value: string) {
        this.#request = value;
      },
    },
    userRequest: {
      enumerable: true,
      configurable: true,
      get(this: NormalModule): string {
        if (this.#userRequest === null) {
          this.#userRequest = binding.normalModuleGetUserRequest(this);
        }
        return this.#userRequest;
      },
      set(this: NormalModule, value: string) {
        this.#userRequest = value;
      },
    },
    rawRequest: {
      enumerable: true,
      configurable: true,
      get(this: NormalModule): string {
        if (this.#rawRequest === null) {
          this.#rawRequest = binding.normalModuleGetRawRequest(this);
        }
        return this.#rawRequest;
      },
      set(this: NormalModule, value: string) {
        this.#rawRequest = value;
      },
    },
    resourceResolveData: {
      enumerable: true,
      configurable: true,
      get(this: NormalModule): Readonly<binding.JsResourceData> | undefined {
        if (this.#resourceResolveData === null) {
          this.#resourceResolveData =
            binding.normalModuleGetResourceResolveData(this);
        }
        return this.#resourceResolveData;
      },
      set(
        this: NormalModule,
        value: Readonly<binding.JsResourceData> | undefined,
      ) {
        this.#resourceResolveData = value;
      },
    },
    loaders: {
      enumerable: true,
      configurable: true,
      get(this: NormalModule): binding.JsLoaderItem[] {
        if (this.#loaders === null) {
          this.#loaders = binding.normalModuleGetLoaders(this);
        }
        return this.#loaders;
      },
      set(this: NormalModule, value: binding.JsLoaderItem[]) {
        this.#loaders = value;
      },
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

  constructor() {
    super();
    Object.defineProperties(this, NormalModule.#properties);
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
