import type { JsExportsInfo } from '@rspack/binding';
import type { RuntimeSpec } from './util/runtime';

import { toJsRuntimeSpec } from './util/runtime';

/**
 * Unused: 0
 * OnlyPropertiesUsed: 1
 * NoInfo: 2
 * Unknown: 3
 * Used: 4
 */
type UsageStateType = 0 | 1 | 2 | 3 | 4;

export class ExportsInfo {
  #inner: JsExportsInfo;

  static __from_binding(binding: JsExportsInfo) {
    return new ExportsInfo(binding);
  }

  private constructor(binding: JsExportsInfo) {
    this.#inner = binding;
  }

  isUsed(runtime: RuntimeSpec): boolean {
    return this.#inner.isUsed(toJsRuntimeSpec(runtime));
  }

  isModuleUsed(runtime: RuntimeSpec): boolean {
    return this.#inner.isModuleUsed(toJsRuntimeSpec(runtime));
  }

  setUsedInUnknownWay(runtime: RuntimeSpec): boolean {
    return this.#inner.setUsedInUnknownWay(toJsRuntimeSpec(runtime));
  }

  getUsed(name: string | string[], runtime: RuntimeSpec): UsageStateType {
    return this.#inner.getUsed(name, toJsRuntimeSpec(runtime));
  }
}
