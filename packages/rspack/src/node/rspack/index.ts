import createDebug from "debug";

import type { RawOptions, ExternalObject, OnLoadContext, OnResolveContext } from "@rspack/binding";
import * as binding from "@rspack/binding";

import type { RspackPlugin } from "./plugin"

const debugRspack = createDebug("rspack");
const debugNapi = createDebug("napi");

binding.initCustomTraceSubscriber();

export type { RawOptions };

interface RspackOptions extends RawOptions {
  plugins?: RspackPlugin[]
}

interface RspackThreadsafeContext<T> {
  readonly callId: number
  readonly inner: T
}

interface RspackThreadsafeResult<T> {
  readonly callId: number
  readonly inner: T
}

const createDummyResult = (callId: number): string => {
  const result: RspackThreadsafeResult<null> = {
    callId,
    inner: null
  }
  return JSON.stringify(result);
}

class Rspack {
  #instance: ExternalObject<any>;

  constructor(options: RspackOptions) {
    const innerOptions: RspackOptions = {
      ...options,
    };

    debugRspack("rspack options", innerOptions);

    const plugins = (innerOptions.plugins || []);

    const onLoad = async (err, value: string): Promise<string> => {
      const context: RspackThreadsafeContext<OnLoadContext> = JSON.parse(value)
      debugNapi("onLoadcontext", context);

      for (const plugin of plugins) {
        const result = await plugin.onLoad(context.inner);
        debugNapi("onLoadResult", result);

        return JSON.stringify({
          callId: context.callId,
          inner: result,
        });
      }

      debugNapi("onLoadResult", null);

      return createDummyResult(context.callId);
    }

    const onResolve = async (err, value: string): Promise<string> => {
      const context: RspackThreadsafeContext<OnResolveContext> = JSON.parse(value)
      debugNapi("onResolveContext", context);

      for (const plugin of plugins) {
        const result = await plugin.onResolve(context.inner);
        debugNapi("onResolveResult", result);

        return JSON.stringify({
          callId: context.callId,
          inner: result,
        });
      }

      debugNapi("onResolveResult", null);

      return createDummyResult(context.callId);
    }

    this.#instance = binding.newRspack(JSON.stringify(options), onLoad, onResolve);
  }

  async build() {
    return binding.build(this.#instance);
  }

  async rebuild(changefile: string) {
    return binding.rebuild(this.#instance, changefile);
  }
}

export { Rspack };
export default Rspack;
