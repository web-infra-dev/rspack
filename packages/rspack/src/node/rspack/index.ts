import createDebug from 'debug';
import type {
  RawOptions,
  ExternalObject,
  OnLoadContext,
  OnResolveContext,
  OnLoadResult,
  OnResolveResult,
} from '@rspack/binding';
import * as binding from '@rspack/binding';

import type { RspackPlugin } from './plugins';

const debugRspack = createDebug('rspack');
const debugNapi = createDebug('napi');

binding.initCustomTraceSubscriber();

export type {
  RawOptions,
  OnLoadContext,
  OnResolveResult,
  OnLoadResult,
  OnResolveContext,
  RspackPlugin,
};

interface RspackOptions extends RawOptions { plugins?: RspackPlugin[] }

interface RspackThreadsafeContext<T> {
  readonly callId: number;
  readonly inner: T;
}

interface RspackThreadsafeResult<T> {
  readonly callId: number;
  readonly inner: T;
}

const createDummyResult = (callId: number): string => {
  const result: RspackThreadsafeResult<null> = { callId, inner: null };
  return JSON.stringify(result);
};

const isNil = (value: unknown): value is null | undefined => {
  return value === null || value === undefined;
};

class Rspack {
  #instance: ExternalObject<any>;
  lazyCompilerMap: Record<string, string>;
  constructor(public options: RspackOptions) {
    const innerOptions: RspackOptions = { ...options };

    debugRspack('rspack options', innerOptions);

    const plugins = innerOptions.plugins || [];
    const isPluginExist = !!plugins.length;

    const onLoad = async (err: Error, value: string): Promise<string> => {
      if (err) {
        throw err;
      }

      const context: RspackThreadsafeContext<OnLoadContext> = JSON.parse(value);

      for (const plugin of plugins) {
        const result = await plugin.onLoad(context.inner);
        debugNapi('onLoadResult', result, 'context', context);

        if (isNil(result)) {
          continue;
        }

        return JSON.stringify({ callId: context.callId, inner: result });
      }

      debugNapi('onLoadResult', null, 'context', context);

      return createDummyResult(context.callId);
    };

    const onResolve = async (err: Error, value: string): Promise<string> => {
      if (err) {
        throw err;
      }

      const context: RspackThreadsafeContext<OnResolveContext> = JSON.parse(
        value,
      );

      for (const plugin of plugins) {
        const result = await plugin.onResolve(context.inner);
        debugNapi('onResolveResult', result, 'context', context);

        if (isNil(result)) {
          continue;
        }

        return JSON.stringify({ callId: context.callId, inner: result });
      }

      debugNapi('onResolveResult', null, 'context', context);

      return createDummyResult(context.callId);
    };

    this.#instance =
      binding.newRspack(
        JSON.stringify(options),
        isPluginExist ? { onloadCallback: onLoad, onresolveCallback: onResolve } : null,
      );
  }

  async build() {
    const map = await binding.build(this.#instance) as unknown as Record<
      string,
      string
    >;
    this.setLazyCompilerMap(map);
    return map;
  }

  async rebuild(changefile: string) {
    const [diff, map] = await binding.rebuild(this.#instance, changefile) as unknown as Record<
      string,
      string
    >[];
    this.setLazyCompilerMap(map);
    return diff;
  }

  setLazyCompilerMap(map) {
    for (const key in map) {
      const value = map[key];
      if (this.options.entries.indexOf(value) > -1) {
        delete map[key];
      }
    }
    this.lazyCompilerMap = map;
  }

  lazyCompileredSet = new Set<string>();

  async lazyBuild(chunkName: string) {
    const filename = this.lazyCompilerMap[chunkName];
    if (filename && !this.lazyCompileredSet.has(filename)) {
      console.log('lazy compiler ', filename);
      this.lazyCompileredSet.add(filename);
      await this.rebuild(filename);
    }
  }
}

export { Rspack };
export default Rspack;
