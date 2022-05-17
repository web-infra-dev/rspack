import type { RawOptions, ExternalObject } from '@rspack/binding';
import * as binding from '@rspack/binding';
binding.initCustomTraceSubscriber();

export type { RawOptions };
class Rspack {
  private _instance: ExternalObject<any>;

  constructor(rawOptions: RawOptions) {
    const options: RawOptions = {
      minify: false,
      ...rawOptions,
    };
    console.log("rawOpts", rawOptions);
    const onLoad = async (err, value) => {
      console.log("[from rust]", value);
      
      return value
    }
    this._instance = binding.newRspack(JSON.stringify(options), onLoad);
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
