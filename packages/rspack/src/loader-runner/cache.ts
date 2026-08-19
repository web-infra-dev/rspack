import type { JsLoaderContext } from '@rspack/binding';

import { isNil, toBuffer } from '../util';
import { createHash } from '../util/createHash';

type LoaderCacheEntry = {
  content: Buffer | null;
  contentIsString: boolean;
  sourceMap?: Buffer;
};

type LoaderCacheApi = {
  get(loaderIndex: number, etag: string): LoaderCacheEntry | null;
  store(loaderIndex: number, etag: string, output: LoaderCacheEntry): void;
};

type LoaderCacheInput = {
  etag: string;
};

function toOwnedBuffer(value: string | Buffer | Uint8Array) {
  if (typeof value === 'string') return Buffer.from(value);
  return Buffer.from(value);
}

function updateHashSegment(
  hash: ReturnType<typeof createHash>,
  label: string,
  value: Buffer,
) {
  const length = Buffer.allocUnsafe(8);
  length.writeBigUInt64LE(BigInt(value.length));
  hash.update(Buffer.from(label));
  hash.update(length);
  hash.update(value);
}

export class LoaderCache {
  readonly #api: LoaderCacheApi;
  readonly #context: JsLoaderContext;
  readonly #workerInputs = new Map<number, LoaderCacheInput>();

  constructor(context: JsLoaderContext) {
    this.#context = context;
    this.#api = (context as any).__internal__loaderCache as LoaderCacheApi;
  }

  begin(
    loaderIndex: number,
    content: Parameters<typeof toBuffer>[0] | null | undefined,
    additionalData: unknown,
  ): LoaderCacheInput | undefined {
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

    const hash = createHash('xxhash64');
    updateHashSegment(hash, 'content', toBuffer(content));
    updateHashSegment(hash, 'options', Buffer.from(loader.optionsCacheKey));
    updateHashSegment(hash, 'version', Buffer.from(loader.loaderVersion));

    return { etag: hash.digest('hex') };
  }

  get(loaderIndex: number, input: LoaderCacheInput) {
    return this.#api.get(loaderIndex, input.etag) ?? undefined;
  }

  store(
    loaderIndex: number,
    input: LoaderCacheInput,
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

    this.#api.store(loaderIndex, input.etag, {
      content: isNil(content) ? null : toOwnedBuffer(content),
      contentIsString,
      sourceMap: sourceMap ? Buffer.from(sourceMap) : undefined,
    });
  }

  workerGet(
    loaderIndex: number,
    content: Parameters<typeof toBuffer>[0] | null | undefined,
    additionalData: unknown,
  ) {
    const input = this.begin(loaderIndex, content, additionalData);
    if (!input) return undefined;
    this.#workerInputs.set(loaderIndex, input);
    const hit = this.get(loaderIndex, input);
    if (!hit) return undefined;
    this.#workerInputs.delete(loaderIndex);
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
    const input = this.#workerInputs.get(loaderIndex);
    if (!input) return;
    this.#workerInputs.delete(loaderIndex);
    this.store(
      loaderIndex,
      input,
      content,
      sourceMap,
      additionalData,
      contentIsString,
    );
  }
}
