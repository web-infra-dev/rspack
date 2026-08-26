import type { JsLoaderContext } from '@rspack/binding';

import { isNil } from '../util';
import {
  type LoaderDependencies,
  LoaderDependenciesState,
} from './dependencies';

type LoaderCacheContent = string | Uint8Array;

export type LoaderCacheEntry = {
  content: LoaderCacheContent | null;
  sourceMap?: Uint8Array;
  addedDependencies: LoaderDependencies;
  removedDependencies: LoaderDependencies;
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
  readonly #dependencies: LoaderDependenciesState;

  constructor(context: JsLoaderContext, dependencies: LoaderDependenciesState) {
    this.#context = context;
    this.#api = (context as any).__internal__loaderCache as LoaderCacheApi;
    this.#dependencies = dependencies;
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

    const hit = this.#api.get(loaderIndex, content);
    if (hit) {
      this.#dependencies.addDependencies(hit.addedDependencies);
    }
    return hit;
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
      addedDependencies: this.#dependencies.added,
      removedDependencies: this.#dependencies.removed,
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
