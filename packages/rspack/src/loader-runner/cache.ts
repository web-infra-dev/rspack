import type { JsLoaderContext } from '@rspack/binding';

import { isNil } from '../util';

type LoaderCacheContent = string | Uint8Array;

export type LoaderCacheEntry = {
  content: LoaderCacheContent | null;
  sourceMap?: Uint8Array;
};

type LoaderCacheApi = {
  get(
    loaderIndex: number,
    content: LoaderCacheContent,
  ): LoaderCacheEntry | null;
  store(loaderIndex: number, output: LoaderCacheEntry): void;
};

export class LoaderCache {
  readonly #api: LoaderCacheApi;
  readonly #context: JsLoaderContext;

  constructor(context: JsLoaderContext) {
    this.#context = context;
    this.#api = (context as any).__internal__loaderCache as LoaderCacheApi;
  }

  get(
    loaderIndex: number,
    content: LoaderCacheContent | null | undefined,
    additionalData: unknown,
  ): LoaderCacheEntry | null | undefined {
    const context = this.#context;
    const loader = context.loaderItems[loaderIndex];
    if (
      !context.cacheable ||
      !loader ||
      isNil(content) ||
      !isNil(additionalData) ||
      Object.keys(context.__internal__parseMeta).length > 0
    ) {
      return undefined;
    }

    return this.#api.get(loaderIndex, content);
  }

  store(
    loaderIndex: number,
    content: LoaderCacheContent | null | undefined,
    sourceMap: Uint8Array | undefined,
    additionalData: unknown,
  ) {
    const context = this.#context;
    if (
      !context.cacheable ||
      !isNil(additionalData) ||
      Object.keys(context.__internal__parseMeta).length > 0
    ) {
      return;
    }

    this.#api.store(loaderIndex, {
      content: isNil(content) ? null : content,
      sourceMap,
    });
  }

  workerGet(
    loaderIndex: number,
    content: LoaderCacheContent | null | undefined,
    additionalData: unknown,
  ) {
    const hit = this.get(loaderIndex, content, additionalData);
    if (!hit) return undefined;
    return hit;
  }

  workerStore(
    loaderIndex: number,
    content: LoaderCacheContent | null | undefined,
    sourceMap: Uint8Array | undefined,
    additionalData: unknown,
  ) {
    this.store(loaderIndex, content, sourceMap, additionalData);
  }
}
