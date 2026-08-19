import type { JsLoaderContext } from '@rspack/binding';

import { isNil, toBuffer } from '../util';
import { createHash } from '../util/createHash';

type LoaderCacheEntry = {
  content: Buffer | null;
  contentIsString: boolean;
  sourceMap?: Buffer;
  additionalData?: Buffer;
  fileDependenciesAdded: string[];
  fileDependenciesRemoved: string[];
  contextDependenciesAdded: string[];
  contextDependenciesRemoved: string[];
  missingDependenciesAdded: string[];
  missingDependenciesRemoved: string[];
  buildDependenciesAdded: string[];
  buildDependenciesRemoved: string[];
  parseMetaUpserted: Record<string, string>;
  parseMetaRemoved: string[];
};

type LoaderCacheApi = {
  get(loaderIndex: number, etag: string): LoaderCacheEntry | null;
  store(loaderIndex: number, etag: string, output: LoaderCacheEntry): void;
};

type LoaderCacheInput = {
  etag: string;
  fileDependencies: string[];
  contextDependencies: string[];
  missingDependencies: string[];
  buildDependencies: string[];
  parseMeta: Record<string, string>;
  sideEffects: number;
};

type LoaderCacheHit = Omit<LoaderCacheEntry, 'additionalData'> & {
  additionalData?: unknown;
};

type DependencyDelta = {
  added: string[];
  removed: string[];
};

function dependencyDelta(
  baseline: string[],
  current: string[],
): DependencyDelta {
  const baselineSet = new Set(baseline);
  const currentSet = new Set(current);
  return {
    added: current.filter((dependency) => !baselineSet.has(dependency)),
    removed: baseline.filter((dependency) => !currentSet.has(dependency)),
  };
}

function applyDependencyDelta(
  dependencies: string[],
  added: string[],
  removed: string[],
) {
  const removedSet = new Set(removed);
  dependencies.splice(
    0,
    dependencies.length,
    ...dependencies.filter((dependency) => !removedSet.has(dependency)),
    ...added,
  );
}

function parseMetaDelta(
  baseline: Record<string, string>,
  current: Record<string, string>,
) {
  const upserted = Object.fromEntries(
    Object.entries(current).filter(([key, value]) => baseline[key] !== value),
  );
  const removed = Object.keys(baseline).filter((key) => !(key in current));
  return { upserted, removed };
}

function toOwnedBuffer(value: string | Buffer | Uint8Array) {
  if (typeof value === 'string') return Buffer.from(value);
  return Buffer.from(value);
}

function serializeAdditionalData(value: unknown): Buffer | null | undefined {
  if (value === undefined) return null;
  try {
    if (!isJsonValue(value, new Set())) return undefined;
    const serialized = JSON.stringify(value);
    return serialized === undefined ? undefined : Buffer.from(serialized);
  } catch {
    return undefined;
  }
}

function isJsonValue(value: unknown, seen: Set<object>): boolean {
  if (
    value === null ||
    typeof value === 'string' ||
    typeof value === 'boolean'
  ) {
    return true;
  }
  if (typeof value === 'number') return Number.isFinite(value);
  if (typeof value !== 'object' || seen.has(value)) return false;

  seen.add(value);
  const isArray = Array.isArray(value);
  const prototype = Object.getPrototypeOf(value);
  if (!isArray && prototype !== Object.prototype && prototype !== null) {
    return false;
  }
  const values = isArray ? value : Object.values(value);
  const result = values.every((item) => isJsonValue(item, seen));
  seen.delete(value);
  return result;
}

function serializeParseMeta(parseMeta: Record<string, string>) {
  return JSON.stringify(
    Object.entries(parseMeta).sort(([left], [right]) =>
      left < right ? -1 : left > right ? 1 : 0,
    ),
  );
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
  #sideEffects = 0;

  constructor(context: JsLoaderContext) {
    this.#context = context;
    this.#api = (context as any).__internal__loaderCache as LoaderCacheApi;
  }

  markSideEffect() {
    this.#sideEffects++;
  }

  begin(
    content: Parameters<typeof toBuffer>[0] | null | undefined,
    sourceMap: Buffer | Uint8Array | undefined,
    additionalData: unknown,
    contentIsString: boolean,
  ): LoaderCacheInput | undefined {
    const context = this.#context;
    if (
      !context.cacheable ||
      isNil(content) ||
      this.#hasModuleBuildSideEffects()
    ) {
      return undefined;
    }

    const serializedAdditionalData = serializeAdditionalData(additionalData);
    if (serializedAdditionalData === undefined) return undefined;

    const hash = createHash('xxhash64');
    updateHashSegment(
      hash,
      contentIsString ? 'string' : 'buffer',
      toBuffer(content),
    );
    if (sourceMap) {
      updateHashSegment(hash, 'source-map', Buffer.from(sourceMap));
    }
    if (serializedAdditionalData) {
      updateHashSegment(hash, 'additional-data', serializedAdditionalData);
    }
    updateHashSegment(
      hash,
      'parse-meta',
      Buffer.from(serializeParseMeta(context.__internal__parseMeta)),
    );

    return {
      etag: hash.digest('hex'),
      fileDependencies: context.fileDependencies.slice(),
      contextDependencies: context.contextDependencies.slice(),
      missingDependencies: context.missingDependencies.slice(),
      buildDependencies: context.buildDependencies.slice(),
      parseMeta: { ...context.__internal__parseMeta },
      sideEffects: this.#sideEffects,
    };
  }

  get(
    loaderIndex: number,
    input: LoaderCacheInput,
  ): LoaderCacheHit | undefined {
    const entry = this.#api.get(loaderIndex, input.etag);
    if (!entry) return undefined;
    let additionalData: unknown;
    try {
      additionalData = entry.additionalData
        ? JSON.parse(Buffer.from(entry.additionalData).toString())
        : undefined;
    } catch {
      return undefined;
    }
    this.#apply(entry);
    return {
      ...entry,
      additionalData,
    };
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
      this.#sideEffects !== input.sideEffects ||
      this.#hasModuleBuildSideEffects()
    ) {
      return;
    }

    const serializedAdditionalData = serializeAdditionalData(additionalData);
    if (serializedAdditionalData === undefined) return;

    const fileDependencies = dependencyDelta(
      input.fileDependencies,
      context.fileDependencies,
    );
    const contextDependencies = dependencyDelta(
      input.contextDependencies,
      context.contextDependencies,
    );
    const missingDependencies = dependencyDelta(
      input.missingDependencies,
      context.missingDependencies,
    );
    const buildDependencies = dependencyDelta(
      input.buildDependencies,
      context.buildDependencies,
    );
    const parseMeta = parseMetaDelta(
      input.parseMeta,
      context.__internal__parseMeta,
    );

    this.#api.store(loaderIndex, input.etag, {
      content: isNil(content) ? null : toOwnedBuffer(content),
      contentIsString,
      sourceMap: sourceMap ? Buffer.from(sourceMap) : undefined,
      additionalData: serializedAdditionalData ?? undefined,
      fileDependenciesAdded: fileDependencies.added,
      fileDependenciesRemoved: fileDependencies.removed,
      contextDependenciesAdded: contextDependencies.added,
      contextDependenciesRemoved: contextDependencies.removed,
      missingDependenciesAdded: missingDependencies.added,
      missingDependenciesRemoved: missingDependencies.removed,
      buildDependenciesAdded: buildDependencies.added,
      buildDependenciesRemoved: buildDependencies.removed,
      parseMetaUpserted: parseMeta.upserted,
      parseMetaRemoved: parseMeta.removed,
    });
  }

  workerGet(
    loaderIndex: number,
    content: Parameters<typeof toBuffer>[0] | null | undefined,
    contentIsString: boolean,
    sourceMap: Buffer | Uint8Array | undefined,
    additionalData: unknown,
  ) {
    const input = this.begin(
      content,
      sourceMap,
      additionalData,
      contentIsString,
    );
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

  #apply(entry: LoaderCacheEntry) {
    const context = this.#context;
    applyDependencyDelta(
      context.fileDependencies,
      entry.fileDependenciesAdded,
      entry.fileDependenciesRemoved,
    );
    applyDependencyDelta(
      context.contextDependencies,
      entry.contextDependenciesAdded,
      entry.contextDependenciesRemoved,
    );
    applyDependencyDelta(
      context.missingDependencies,
      entry.missingDependenciesAdded,
      entry.missingDependenciesRemoved,
    );
    applyDependencyDelta(
      context.buildDependencies,
      entry.buildDependenciesAdded,
      entry.buildDependenciesRemoved,
    );
    for (const key of entry.parseMetaRemoved) {
      delete context.__internal__parseMeta[key];
    }
    Object.assign(context.__internal__parseMeta, entry.parseMetaUpserted);
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
