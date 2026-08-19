import type { JsLoaderContext } from '@rspack/binding';

import { isNil, toBuffer } from '../util';

export type LoaderCacheEntry = {
  content: Buffer | null;
  contentIsString: boolean;
  sourceMap?: Buffer;
};

type LoaderCacheApi = {
  get(
    loaderIndex: number,
    content: Buffer,
    sourceMap?: Buffer,
  ): LoaderCacheEntry | null;
  store(loaderIndex: number, output: LoaderCacheEntry): void;
};

function toOwnedBuffer(value: string | Buffer | Uint8Array) {
  if (typeof value === 'string') return Buffer.from(value);
  return Buffer.from(value);
}

export class LoaderCache {
  readonly #api: LoaderCacheApi;
  readonly #context: JsLoaderContext;

  constructor(context: JsLoaderContext) {
    this.#context = context;
    this.#api = (context as any).__internal__loaderCache as LoaderCacheApi;
  }

  get(
    loaderIndex: number,
    content: Parameters<typeof toBuffer>[0] | null | undefined,
    sourceMap: Buffer | Uint8Array | undefined,
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

    return this.#api.get(
      loaderIndex,
      toBuffer(content),
      sourceMap ? Buffer.from(sourceMap) : undefined,
    );
  }

  store(
    loaderIndex: number,
    content: Parameters<typeof toBuffer>[0] | null | undefined,
    sourceMap: Buffer | Uint8Array | undefined,
    additionalData: unknown,
    contentIsString: boolean,
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
      content: isNil(content) ? null : toOwnedBuffer(content),
      contentIsString,
      sourceMap: sourceMap ? Buffer.from(sourceMap) : undefined,
    });
  }

  workerGet(
    loaderIndex: number,
    content: Parameters<typeof toBuffer>[0] | null | undefined,
    sourceMap: Buffer | Uint8Array | undefined,
    additionalData: unknown,
  ) {
    const hit = this.get(loaderIndex, content, sourceMap, additionalData);
    if (!hit) return undefined;
    return {
      ...hit,
      content: hit.content ? Buffer.from(hit.content) : null,
      sourceMap: hit.sourceMap ? Buffer.from(hit.sourceMap) : undefined,
    };
  }

  workerStore(
    loaderIndex: number,
    content: Parameters<typeof toBuffer>[0] | null | undefined,
    contentIsString: boolean,
    sourceMap: Buffer | Uint8Array | undefined,
    additionalData: unknown,
  ) {
    this.store(
      loaderIndex,
      content,
      sourceMap,
      additionalData,
      contentIsString,
    );
  }
}
