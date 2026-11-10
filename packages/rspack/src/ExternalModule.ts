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
  #userRequest: string | null = null;

  static readonly #properties: PropertyDescriptorMap = {
    userRequest: {
      enumerable: true,
      configurable: true,
      get(this: ExternalModule): string {
        if (this.#userRequest === null) {
          this.#userRequest = binding.externalModuleGetUserRequest(this);
        }
        return this.#userRequest;
      },
      set(this: ExternalModule, value: string) {
        this.#userRequest = value;
      },
    },
  };

  declare readonly userRequest: string;

  constructor() {
    super();
    Object.defineProperties(this, ExternalModule.#properties);
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
