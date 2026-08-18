import type { JsLoaderContext } from '@rspack/binding';

import { isNil, toBuffer } from '../util';

export type LoaderCacheData = {
  content: Buffer | null;
  contentIsString: boolean;
  sourceMap?: Buffer;
  additionalData?: unknown;
  additionalDataCacheKey?: string;
  fileDependencies: string[];
  contextDependencies: string[];
  missingDependencies: string[];
  buildDependencies: string[];
  parseMeta: Record<string, string>;
  cacheable: boolean;
  hasUnhandledSideEffects: boolean;
};

type LoaderCacheApi = {
  get(cacheKey: string, input: LoaderCacheData): LoaderCacheData | null;
  store(
    cacheKey: string,
    input: LoaderCacheData,
    output: LoaderCacheData,
  ): string | null;
};

type WorkerCacheInput = {
  data: LoaderCacheData;
  sideEffects: number;
};

function replaceArray<T>(target: T[], source: T[]) {
  target.splice(0, target.length, ...source);
}

function toOwnedBuffer(value: string | Buffer | Uint8Array) {
  if (typeof value === 'string') return Buffer.from(value);
  return Buffer.from(value);
}

export class LoaderCache {
  readonly #api: LoaderCacheApi;
  readonly #context: JsLoaderContext;
  readonly #workerInputs = new Map<number, WorkerCacheInput>();
  #additionalDataCacheKey: string | undefined;
  #sideEffects = 0;

  constructor(context: JsLoaderContext) {
    this.#context = context;
    this.#api = (context as any).__internal__loaderCache as LoaderCacheApi;
  }

  get sideEffects() {
    return this.#sideEffects;
  }

  markSideEffect() {
    this.#sideEffects++;
  }

  snapshot(
    content: Parameters<typeof toBuffer>[0] | null | undefined,
    sourceMap: Buffer | Uint8Array | undefined,
    additionalData: unknown,
    contentIsString: boolean,
    hasUnhandledSideEffects = false,
  ): LoaderCacheData {
    const context = this.#context;
    return {
      content: isNil(content) ? null : toOwnedBuffer(content),
      contentIsString,
      sourceMap: sourceMap ? Buffer.from(sourceMap) : undefined,
      additionalData,
      additionalDataCacheKey: this.#additionalDataCacheKey,
      fileDependencies: context.fileDependencies.slice(),
      contextDependencies: context.contextDependencies.slice(),
      missingDependencies: context.missingDependencies.slice(),
      buildDependencies: context.buildDependencies.slice(),
      parseMeta: { ...context.__internal__parseMeta },
      cacheable: context.cacheable,
      hasUnhandledSideEffects:
        hasUnhandledSideEffects || this.#hasModuleBuildSideEffects(),
    };
  }

  get(cacheKey: string, input: LoaderCacheData) {
    const hit = this.#api.get(cacheKey, input);
    if (hit) this.#apply(hit);
    return hit ?? undefined;
  }

  store(
    cacheKey: string,
    input: LoaderCacheData,
    output: LoaderCacheData,
    inputAdditionalData: unknown,
    outputAdditionalData: unknown,
  ) {
    const storedKey = this.#api.store(cacheKey, input, output);
    this.#additionalDataCacheKey =
      storedKey ??
      (outputAdditionalData === inputAdditionalData
        ? input.additionalDataCacheKey
        : undefined);
    return storedKey ?? undefined;
  }

  invalidateAdditionalData(input: unknown, output: unknown) {
    if (input !== output) this.#additionalDataCacheKey = undefined;
  }

  invalidate() {
    this.#additionalDataCacheKey = undefined;
  }

  workerGet(
    loaderIndex: number,
    cacheKey: string,
    content: Parameters<typeof toBuffer>[0] | null | undefined,
    contentIsString: boolean,
    sourceMap: Buffer | Uint8Array | undefined,
    additionalData: unknown,
  ) {
    const data = this.snapshot(
      content,
      sourceMap,
      additionalData,
      contentIsString,
    );
    this.#workerInputs.set(loaderIndex, {
      data,
      sideEffects: this.#sideEffects,
    });
    const hit = this.get(cacheKey, data);
    if (!hit) return undefined;
    return {
      ...hit,
      content: hit.content ? Buffer.from(hit.content) : null,
      sourceMap: hit.sourceMap ? Buffer.from(hit.sourceMap) : undefined,
    };
  }

  workerStore(
    loaderIndex: number,
    cacheKey: string,
    content: Parameters<typeof toBuffer>[0] | null | undefined,
    contentIsString: boolean,
    sourceMap: Buffer | Uint8Array | undefined,
    additionalData: unknown,
  ) {
    const input = this.#workerInputs.get(loaderIndex);
    if (!input) return undefined;
    this.#workerInputs.delete(loaderIndex);
    const output = this.snapshot(
      content,
      sourceMap,
      additionalData,
      contentIsString,
      this.#sideEffects !== input.sideEffects,
    );
    return this.store(
      cacheKey,
      input.data,
      output,
      input.data.additionalData,
      additionalData,
    );
  }

  #apply(data: LoaderCacheData) {
    const context = this.#context;
    replaceArray(context.fileDependencies, data.fileDependencies);
    replaceArray(context.contextDependencies, data.contextDependencies);
    replaceArray(context.missingDependencies, data.missingDependencies);
    replaceArray(context.buildDependencies, data.buildDependencies);
    for (const key of Object.keys(context.__internal__parseMeta)) {
      delete context.__internal__parseMeta[key];
    }
    Object.assign(context.__internal__parseMeta, data.parseMeta);
    this.#additionalDataCacheKey = data.additionalDataCacheKey;
  }

  #hasModuleBuildSideEffects() {
    const buildInfo = this.#context._module.buildInfo;
    return (
      Object.keys(buildInfo.assets).length > 0 ||
      Object.keys(buildInfo).some(
        (key) =>
          key !== 'assets' &&
          key !== 'fileDependencies' &&
          key !== 'contextDependencies' &&
          key !== 'missingDependencies' &&
          key !== 'buildDependencies',
      )
    );
  }
}
