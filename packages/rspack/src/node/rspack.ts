import type { BundleOptions, ExternalObject } from '@rspack/binding';
import * as binding from '@rspack/binding';

export type { BundleOptions };

class Rspack {
  private _instance: ExternalObject<any>;

  constructor(rawOptions: BundleOptions) {
    const options: BundleOptions = {
      minify: false,
      ...rawOptions,
    };
    this._instance = binding.newRspack(JSON.stringify(options));
  }

  async build() {
    return binding.build(this._instance);
  }
  async rebuild(changefile: string) {
    return binding.rebuild(this._instance, changefile);
  }
}

export { Rspack };
export default Rspack;
