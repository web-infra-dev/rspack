import binding from '@rspack/binding';
import * as liteTapable from '@rspack/lite-tapable';
import type { Chunk } from './Chunk';
import { type Compilation, getOrCreateCompilationHooks } from './Compilation';
import { Module } from './Module';
import { COMPILATION_HOOKS_MAP_SYMBOL } from './util/symbols';

export type ExternalModuleCompilationHooks = {
  chunkCondition: liteTapable.SyncBailHook<
    [Chunk, Compilation],
    boolean | undefined
  >;
};

export class ExternalModule extends Module {
  static readonly #properties: PropertyDescriptorMap = {
    userRequest: {
      enumerable: true,
      configurable: true,
      writable: true,
      value: undefined,
    },
    ...Module._properties,
  };

  declare readonly userRequest: string;

  constructor(data?: binding.JsModuleData) {
    super(data, ExternalModule.#properties);
    if (data?.userRequest !== undefined) this.userRequest = data.userRequest;
  }

  static getCompilationHooks(
    compilation: Compilation,
  ): ExternalModuleCompilationHooks {
    if (!(COMPILATION_HOOKS_MAP_SYMBOL in compilation)) {
      throw new TypeError(
        "The 'compilation' argument must be an instance of Compilation",
      );
    }

    // The binding object is shared when @rspack/core is evaluated more than
    // once. A JS constructor is local to its core instance, so it is not a
    // suitable key for hooks that must work across those instances.
    return getOrCreateCompilationHooks(compilation, binding, () => ({
      chunkCondition: new liteTapable.SyncBailHook(['chunk', 'compilation']),
    }));
  }
}
