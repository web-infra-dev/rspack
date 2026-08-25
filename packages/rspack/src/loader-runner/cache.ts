import type { JsLoaderContext } from '@rspack/binding';

import { isNil } from '../util';

type LoaderCacheContent = string | Uint8Array;

export type LoaderCacheEntry = {
  content: LoaderCacheContent | null;
  sourceMap?: Uint8Array;
  dependencyContext: DependencyContext;
  dependencyContextAppendOnly: boolean;
};

type DependencyContext = JsLoaderContext['dependencyContext'];

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
  readonly #pendingDependencyContexts: Array<DependencyContext | undefined>;

  constructor(context: JsLoaderContext) {
    this.#context = context;
    this.#api = (context as any).__internal__loaderCache as LoaderCacheApi;
    this.#pendingDependencyContexts = new Array(context.loaderItems.length);
  }

  get(
    loaderIndex: number,
    content: LoaderCacheContent | null | undefined,
    additionalData: unknown,
  ): LoaderCacheEntry | null | undefined {
    const context = this.#context;
    const dependencyContext = context.dependencyContext;
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
      dependencyContext.fileDependencies.push(
        ...hit.dependencyContext.fileDependencies,
      );
      dependencyContext.contextDependencies.push(
        ...hit.dependencyContext.contextDependencies,
      );
      dependencyContext.buildDependencies.push(
        ...hit.dependencyContext.buildDependencies,
      );
      this.#pendingDependencyContexts[loaderIndex] = undefined;
    } else {
      this.#pendingDependencyContexts[loaderIndex] = {
        fileDependencies: dependencyContext.fileDependencies.slice(),
        contextDependencies: dependencyContext.contextDependencies.slice(),
        missingDependencies: dependencyContext.missingDependencies.slice(),
        buildDependencies: dependencyContext.buildDependencies.slice(),
      };
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
    const dependencyContext = context.dependencyContext;
    if (
      !context.cacheable ||
      !isNil(additionalData) ||
      Object.keys(context.__internal__parseMeta).length > 0
    ) {
      return;
    }

    const previous = this.#pendingDependencyContexts[loaderIndex];
    this.#pendingDependencyContexts[loaderIndex] = undefined;
    if (!previous) return;

    const isUnchangedPrefix = (current: string[], before: string[]) =>
      current.length >= before.length &&
      before.every((dependency, index) => current[index] === dependency);
    const dependencyContextAppendOnly = (
      [
        'fileDependencies',
        'contextDependencies',
        'missingDependencies',
        'buildDependencies',
      ] as const
    ).every((key) => isUnchangedPrefix(dependencyContext[key], previous[key]));
    const added = (key: keyof DependencyContext) =>
      dependencyContextAppendOnly
        ? dependencyContext[key].slice(previous[key].length)
        : [];

    this.#api.store(loaderIndex, {
      content: isNil(content) ? null : content,
      sourceMap,
      dependencyContext: {
        fileDependencies: added('fileDependencies'),
        contextDependencies: added('contextDependencies'),
        missingDependencies: added('missingDependencies'),
        buildDependencies: added('buildDependencies'),
      },
      dependencyContextAppendOnly,
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
