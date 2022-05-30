import path from 'path';
import type {
  ExternalObject,
  OnLoadContext,
  OnLoadResult,
  OnResolveContext,
  OnResolveResult,
  RspackInternal,
  ResolveOptions,
  ResolveResult,
} from '@rspack/binding';
import * as binding from '@rspack/binding';

import type { RspackOptions } from '..';
import { debugNapi } from '..';

export interface RspackPlugin {
  name: string;
  buildStart?(this: RspackPluginContext): Promise<void>;
  load?(this: RspackPluginContext, id: string): Promise<OnLoadResult | void>;
  resolve?(this: RspackPluginContext, source: string, importer: string | undefined): Promise<OnResolveResult | void>;
  buildEnd?(this: RspackPluginContext): Promise<void>;
}

interface RspackThreadsafeContext<T> {
  readonly id: number;
  readonly inner: T;
}

interface RspackThreadsafeResult<T> {
  readonly id: number;
  readonly inner: T;
}

const createDummyResult = (id: number): string => {
  const result: RspackThreadsafeResult<null> = {
    id,
    inner: null,
  };
  return JSON.stringify(result);
};

const isNil = (value: unknown): value is null | undefined => {
  return value === null || value === undefined;
};

class RspackPluginContext {
  constructor(private factory: RspackPluginFactory) {}

  resolve(source: string, importer: string | undefined, resolveOptions?: ResolveOptions): ResolveResult {
    return binding.resolve(this.factory._rspack, source, {
      ...resolveOptions,
      resolveDir: importer ? path.dirname(importer) : this.factory.options.root,
    });
  }
}

export class RspackPluginFactory {
  _rspack: ExternalObject<RspackInternal>;

  private pluginContext: RspackPluginContext;

  constructor(public plugins: RspackPlugin[], public options: RspackOptions) {
    this.pluginContext = new RspackPluginContext(this);

    this.buildStart = this.buildStart.bind(this);
    this.buildEnd = this.buildEnd.bind(this);
    this.load = this.load.bind(this);
    this.resolve = this.resolve.bind(this);
  }

  setRspackInstance(rspack: ExternalObject<RspackInternal>) {
    this._rspack = rspack;
  }

  async buildStart(err: Error, value: string): Promise<string> {
    if (err) {
      throw err;
    }

    const context: RspackThreadsafeContext<void> = JSON.parse(value);

    await Promise.all(this.plugins.map((plugin) => plugin.buildStart?.bind(this.pluginContext)?.()));

    return createDummyResult(context.id);
  }

  async buildEnd(err: Error, value: string): Promise<string> {
    if (err) {
      throw err;
    }

    const context: RspackThreadsafeContext<void> = JSON.parse(value);

    await Promise.all(this.plugins.map((plugin) => plugin.buildEnd?.bind(this.pluginContext)?.()));

    return createDummyResult(context.id);
  }

  async load(err: Error, value: string): Promise<string> {
    if (err) {
      throw err;
    }

    const context: RspackThreadsafeContext<OnLoadContext> = JSON.parse(value);

    for (const plugin of this.plugins) {
      const { id } = context.inner;
      const result = await plugin.load?.(id);
      debugNapi('onLoadResult', result, 'context', context);

      if (isNil(result)) {
        continue;
      }

      return JSON.stringify({
        id: context.id,
        inner: result,
      });
    }

    debugNapi('onLoadResult', null, 'context', context);

    return createDummyResult(context.id);
  }

  async resolve(err: Error, value: string): Promise<string> {
    if (err) {
      throw err;
    }

    const context: RspackThreadsafeContext<OnResolveContext> = JSON.parse(value);

    for (const plugin of this.plugins) {
      const { importer, importee } = context.inner;
      const result = await plugin.resolve?.bind(this.pluginContext)?.(importee, importer);
      debugNapi('onResolveResult', result, 'context', context);

      if (isNil(result)) {
        continue;
      }

      return JSON.stringify({
        id: context.id,
        inner: result,
      });
    }

    debugNapi('onResolveResult', null, 'context', context);
    return createDummyResult(context.id);
  }
}
