import binding from '@rspack/binding';
import type { Source } from 'webpack-sources';
import type { ResourceData } from './Resolver';
import { SourceAdapter } from './util/source';
// patch buildInfo
import './BuildInfo';
import { MODULE_IDENTIFIER_SYMBOL } from './util/symbols';

export type ResourceDataWithData = ResourceData & {
  data?: Record<string, any>;
};
export type ContextInfo = binding.ContextInfo;
export type ResolveData = binding.JsResolveData & {
  fileDependencies: string[];
  contextDependencies: string[];
  missingDependencies: string[];
};

export class ContextModuleFactoryBeforeResolveData {
  #inner: binding.JsContextModuleFactoryBeforeResolveData;

  declare context: string;
  declare request: string;
  declare regExp: RegExp | undefined;
  declare recursive: boolean;

  static __from_binding(
    binding: binding.JsContextModuleFactoryBeforeResolveData,
  ) {
    return new ContextModuleFactoryBeforeResolveData(binding);
  }

  static __to_binding(
    data: ContextModuleFactoryBeforeResolveData,
  ): binding.JsContextModuleFactoryBeforeResolveData {
    return data.#inner;
  }

  private constructor(
    binding: binding.JsContextModuleFactoryBeforeResolveData,
  ) {
    this.#inner = binding;

    Object.defineProperties(this, {
      context: {
        enumerable: true,
        get(): string {
          return binding.context;
        },
        set(val: string) {
          binding.context = val;
        },
      },
      request: {
        enumerable: true,
        get(): string {
          return binding.request;
        },
        set(val: string) {
          binding.request = val;
        },
      },
      regExp: {
        enumerable: true,
        get(): RegExp | undefined {
          return binding.regExp;
        },
        set(val: RegExp | undefined) {
          binding.regExp = val;
        },
      },
      recursive: {
        enumerable: true,
        get(this: ContextModuleFactoryAfterResolveData): boolean {
          return binding.recursive;
        },
        set(val: boolean) {
          binding.recursive = val;
        },
      },
    });
  }
}

export type ContextModuleFactoryBeforeResolveResult =
  false | ContextModuleFactoryBeforeResolveData;

export class ContextModuleFactoryAfterResolveData {
  #inner: binding.JsContextModuleFactoryAfterResolveData;

  declare resource: string;
  declare context: string;
  declare request: string;
  declare regExp: RegExp | undefined;
  declare recursive: boolean;
  declare readonly dependencies: binding.Dependency[];

  static __from_binding(
    binding: binding.JsContextModuleFactoryAfterResolveData,
  ) {
    return new ContextModuleFactoryAfterResolveData(binding);
  }

  static __to_binding(
    data: ContextModuleFactoryAfterResolveData,
  ): binding.JsContextModuleFactoryAfterResolveData {
    return data.#inner;
  }

  private constructor(binding: binding.JsContextModuleFactoryAfterResolveData) {
    this.#inner = binding;

    Object.defineProperties(this, {
      resource: {
        enumerable: true,
        get(): string {
          return binding.resource;
        },
        set(val: string) {
          binding.resource = val;
        },
      },
      context: {
        enumerable: true,
        get(): string {
          return binding.context;
        },
        set(val: string) {
          binding.context = val;
        },
      },
      request: {
        enumerable: true,
        get(): string {
          return binding.request;
        },
        set(val: string) {
          binding.request = val;
        },
      },
      regExp: {
        enumerable: true,
        get(): RegExp | undefined {
          return binding.regExp;
        },
        set(val: RegExp | undefined) {
          binding.regExp = val;
        },
      },
      recursive: {
        enumerable: true,
        get(): boolean {
          return binding.recursive;
        },
        set(val: boolean) {
          binding.recursive = val;
        },
      },
      dependencies: {
        enumerable: true,
        get(): binding.Dependency[] {
          return binding.dependencies;
        },
      },
    });
  }
}

export type ContextModuleFactoryAfterResolveResult =
  false | ContextModuleFactoryAfterResolveData;

export class Module {
  // Shared functions use the call receiver; no descriptor captures a Module instance.
  protected static readonly _properties: PropertyDescriptorMap = {
    type: {
      enumerable: true,
      configurable: true,
      writable: true,
      value: undefined,
    },
    context: {
      enumerable: true,
      configurable: true,
      get(this: Module): string | undefined {
        return binding.moduleGetContext(this);
      },
    },
    layer: {
      enumerable: true,
      configurable: true,
      get(this: Module): string | undefined {
        return binding.moduleGetLayer(this);
      },
    },
    useSourceMap: {
      enumerable: true,
      configurable: true,
      get(this: Module): boolean {
        return binding.moduleGetUseSourceMap(this);
      },
    },
    useSimpleSourceMap: {
      enumerable: true,
      configurable: true,
      get(this: Module): boolean {
        return binding.moduleGetUseSimpleSourceMap(this);
      },
    },
    factoryMeta: {
      enumerable: true,
      configurable: true,
      get(this: Module): binding.JsFactoryMeta {
        return binding.moduleGetFactoryMeta(this);
      },
      set(this: Module, value: binding.JsFactoryMeta) {
        binding.moduleSetFactoryMeta(this, value);
      },
    },
    buildInfo: {
      enumerable: true,
      configurable: true,
      get(this: Module): binding.BuildInfo {
        return binding.moduleGetBuildInfo(this);
      },
      set(this: Module, value: binding.BuildInfo) {
        binding.moduleSetBuildInfo(this, value);
      },
    },
  };

  declare readonly context: string | undefined;
  declare readonly layer: string | undefined;
  declare readonly useSourceMap: boolean;
  declare readonly useSimpleSourceMap: boolean;
  declare factoryMeta: binding.JsFactoryMeta;
  declare buildInfo: binding.BuildInfo;

  declare readonly type: string;
  declare buildMeta: Record<string, any>;

  constructor(data?: binding.JsModuleData, properties = Module._properties) {
    Object.defineProperties(this, properties);
    this.buildMeta = {};
    if (data) {
      Object.defineProperty(this, 'type', { value: data.moduleType });
      Object.defineProperty(this, MODULE_IDENTIFIER_SYMBOL, {
        value: data.identifier,
        configurable: true,
      });
    }
  }

  identifier(): string {
    return (this as unknown as Record<symbol, string>)[
      MODULE_IDENTIFIER_SYMBOL
    ];
  }

  get blocks(): binding.AsyncDependenciesBlock[] {
    return binding.moduleGetBlocks(this);
  }

  get dependencies(): binding.Dependency[] {
    return binding.moduleGetDependencies(this);
  }

  readableIdentifier(): string {
    return binding.moduleGetReadableIdentifier(this);
  }

  originalSource(): Source | null {
    const source = this._originalSource();
    return source ? SourceAdapter.fromBinding(source) : null;
  }

  _originalSource(): binding.JsSourceWithLazyMap | undefined {
    return binding.moduleGetOriginalSource(this);
  }

  nameForCondition(): string | undefined {
    return binding.moduleGetNameForCondition(this);
  }

  size(ty?: string | null): number {
    return binding.moduleSize(this, ty);
  }

  libIdent(options: binding.JsLibIdentOptions): string | null {
    return binding.moduleLibIdent(this, options);
  }

  emitFile(
    filename: string,
    source: Source,
    assetInfo?: binding.AssetInfo,
  ): void {
    this._emitFile(filename, SourceAdapter.toBinding(source), assetInfo);
  }

  _emitFile(
    filename: string,
    source: binding.JsSourceFromJs,
    assetInfo?: binding.AssetInfo,
  ): void {
    binding.moduleEmitFile(this, filename, source, assetInfo);
  }
}

// Preserve the native prototypes' enumerable public methods and dependency
// accessor. These descriptors are installed once, and shared through inheritance.
for (const name of [
  'identifier',
  'originalSource',
  'emitFile',
  'readableIdentifier',
  'nameForCondition',
  'size',
  'libIdent',
  'dependencies',
]) {
  Object.defineProperty(Module.prototype, name, { enumerable: true });
}
