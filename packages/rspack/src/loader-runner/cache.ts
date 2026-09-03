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
  parseMeta: Record<string, string>;
};

export type WorkerCacheResult =
  | { type: 'disabled' }
  | { type: 'miss' }
  | { type: 'hit'; entry: LoaderCacheEntry };

type LoaderCacheApi = {
  get(
    loaderIndex: number,
    content: LoaderCacheContent,
    existing: LoaderDependencies,
  ): Promise<LoaderCacheEntry | null>;
  store(loaderIndex: number, output: LoaderCacheEntry): Promise<void>;
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

  async get(
    loaderIndex: number,
    content: LoaderCacheContent | null | undefined,
    additionalData: unknown,
  ): Promise<LoaderCacheEntry | null | undefined> {
    const context = this.#context;
    const loader = context.loaderItems[loaderIndex];
    if (
      !context.cacheable ||
      !loader ||
      isNil(content) ||
      !isNil(additionalData) ||
      Object.keys(context.__internal__parseMeta).length > 0 ||
      this.#dependencies.contextDependencies().length > 0 ||
      this.#dependencies.missingDependencies().length > 0
    ) {
      return undefined;
    }

    const hit = await this.#api.get(
      loaderIndex,
      content,
      this.#dependencies.existing,
    );
    if (hit) {
      this.#dependencies.addDependencies(hit.addedDependencies);
      Object.assign(context.__internal__parseMeta, hit.parseMeta);
    }
    return hit;
  }

  async store(
    loaderIndex: number,
    content: LoaderCacheContent | null | undefined,
    sourceMap: Uint8Array | undefined,
    additionalData: unknown,
  ) {
    const context = this.#context;
    if (!context.cacheable || !isNil(additionalData)) {
      return;
    }

    await this.#api.store(loaderIndex, {
      content: isNil(content) ? null : content,
      sourceMap,
      addedDependencies: this.#dependencies.added,
      removedDependencies: this.#dependencies.removed,
      parseMeta: { ...context.__internal__parseMeta },
    });
  }

  async workerGet(
    loaderIndex: number,
    content: LoaderCacheContent | null | undefined,
    additionalData: unknown,
  ): Promise<WorkerCacheResult> {
    const hit = await this.get(loaderIndex, content, additionalData);
    if (hit === undefined) return { type: 'disabled' };
    if (hit === null) return { type: 'miss' };
    return { type: 'hit', entry: hit };
  }

  async workerStore(
    loaderIndex: number,
    content: LoaderCacheContent | null | undefined,
    sourceMap: Uint8Array | undefined,
    additionalData: unknown,
  ) {
    await this.store(loaderIndex, content, sourceMap, additionalData);
  }
}
