import createDebug from "debug";

import type { RawOptions, ExternalObject, OnLoadContext, OnResolveContext, OnLoadResult, OnResolveResult } from "@rspack/binding";
import * as binding from "@rspack/binding";

import type { RspackPlugin } from "./plugin";

const debugRspack = createDebug("rspack");
const debugNapi = createDebug("napi");

binding.initCustomTraceSubscriber();

export type { RawOptions, OnLoadContext, OnResolveResult, OnLoadResult, OnResolveContext };

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

const isNil = (value: unknown): value is null | undefined => {
  return value === null || value === undefined
}

class Rspack {
  #instance: ExternalObject<any>;

  constructor(options: RspackOptions) {
    const innerOptions: RspackOptions = {
      ...options,
    };

    debugRspack("rspack options", innerOptions);

    const plugins = (innerOptions.plugins || []);
    const isPluginExist = !!plugins.length;

    const onLoad = async (err: Error, value: string): Promise<string> => {
      if (err) {
        throw err
      }

      const context: RspackThreadsafeContext<OnLoadContext> = JSON.parse(value);
      debugNapi("onLoadcontext", context);

      for (const plugin of plugins) {
        const result = await plugin.onLoad(context.inner);
        debugNapi("onLoadResult", result);

        if(isNil(result)) {
          continue;
        }

        return JSON.stringify({
          callId: context.callId,
          inner: result,
        });
      }

      debugNapi("onLoadResult", null);

      return createDummyResult(context.callId);
    }

    const onResolve = async (err: Error, value: string): Promise<string> => {
      if (err) {
        throw err
      }

      const context: RspackThreadsafeContext<OnResolveContext> = JSON.parse(value);
      debugNapi("onResolveContext", context);

      for (const plugin of plugins) {
        const result = await plugin.onResolve(context.inner);
        debugNapi("onResolveResult", result);

        if(isNil(result)) {
          continue;
        }

        return JSON.stringify({
          callId: context.callId,
          inner: result,
        });
      }

      debugNapi("onResolveResult", null);

      return createDummyResult(context.callId);
    }

    this.#instance = binding.newRspack(JSON.stringify(options), isPluginExist ? {
      onloadCallback: onLoad,
      onresolveCallback: onResolve
    } : null );
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
