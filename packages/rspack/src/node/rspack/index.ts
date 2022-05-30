import createDebug from 'debug';
import type {
  RawOptions,
  ExternalObject,
  OnLoadContext,
  OnResolveContext,
  OnLoadResult,
  OnResolveResult,
  RawOutputOptions,
} from '@rspack/binding';
import * as binding from '@rspack/binding';

import type { RspackPlugin } from './plugins';
import { RspackPluginFactory } from './plugins';

export const debugRspack = createDebug('rspack');
export const debugNapi = createDebug('napi');

binding.initCustomTraceSubscriber();

export type { RawOptions, OnLoadContext, OnResolveResult, OnLoadResult, OnResolveContext, RspackPlugin };

type UnionOmit<T, U> = Omit<T, keyof U> & U;

export type RspackRawOptions = UnionOmit<
  RawOptions,
  {
    output?: UnionOmit<
      RawOutputOptions,
      {
        sourceMap?: 'linked' | 'external' | 'inline' | boolean;
      }
    >;
  }
>;

export interface RspackOptions extends RawOptions {
  plugins?: RspackPlugin[];
}

class Rspack {
  #instance: ExternalObject<any>;
  lazyCompilerMap: Record<string, string>;
  constructor(public options: RspackOptions) {
    const { plugins = [], ...innerOptions } = options;
    debugRspack('rspack options', innerOptions);

    const isPluginExist = !!plugins.length;

    console.log('raw options', options);

    const pluginFactory = new RspackPluginFactory(plugins, options);

    this.#instance = binding.newRspack(
      JSON.stringify(options),
      (() => {
        if (isPluginExist) {
          return {
            loadCallback: pluginFactory.load,
            resolveCallback: pluginFactory.resolve,
            buildStartCallback: pluginFactory.buildStart,
            buildEndCallback: pluginFactory.buildEnd,
          };
        }
        return null;
      })()
    );

    pluginFactory.setRspackInstance(this.#instance);
  }

  async build() {
    const map = await binding.build(this.#instance);
    this.setLazyCompilerMap(map);
    return map;
  }

  async rebuild(changedFile: string[]) {
    const [diff, map] = await binding.rebuild(this.#instance, changedFile);
    this.setLazyCompilerMap(map);
    return diff;
  }

  setLazyCompilerMap(map) {
    for (const key in map) {
      const value = map[key];
      if (Object.values(this.options.entries).indexOf(value) > -1) {
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
      await this.rebuild([filename]);
    }
  }
}

export { Rspack };
export default Rspack;
