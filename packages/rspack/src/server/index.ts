import * as binding from '@rspack/binding';
import type { ExternalObject, RawOptions, RspackInternal } from "@rspack/binding"

class Rspack {
  #instance: ExternalObject<RspackInternal>;

  constructor(public options: RawOptions) {
    this.#instance = binding.newRspack(
      JSON.stringify(options),
    );
  }

  async build() {
    const stats = await binding.build(this.#instance);
    return stats;
  }

  async rebuild() {
    const stats = await binding.rebuild(this.#instance);
    return stats;
  }
}

export { Rspack };
export default Rspack;