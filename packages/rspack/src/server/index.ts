import * as binding from '@rspack/binding';
import * as Config from '../config';
import type { ExternalObject, RspackInternal } from "@rspack/binding"
import type { RspackOptions } from '../config';

class Rspack {
  #instance: ExternalObject<RspackInternal>;

  constructor(public options: RspackOptions) {
    this.#instance = binding.newRspack(
      JSON.stringify(Config.User2Native(options))
    )
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