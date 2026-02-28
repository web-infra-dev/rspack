import util from 'node:util';
import { Chunk, type ChunkGroup } from '@rspack/binding';

Object.defineProperty(Chunk.prototype, 'files', {
  enumerable: true,
  configurable: true,
  get(this: Chunk) {
    return new Set(this._files);
  },
});

Object.defineProperty(Chunk.prototype, 'runtime', {
  enumerable: true,
  configurable: true,
  get(this: Chunk) {
    return new Set(this._runtime);
  },
});

Object.defineProperty(Chunk.prototype, 'auxiliaryFiles', {
  enumerable: true,
  configurable: true,
  get(this: Chunk) {
    return new Set(this._auxiliaryFiles);
  },
});

Object.defineProperty(Chunk.prototype, 'groupsIterable', {
  enumerable: true,
  configurable: true,
  get(this: Chunk) {
    return new Set(this._groupsIterable);
  },
});

interface ChunkMaps {
  hash: Record<string | number, string>;
  contentHash: Record<string | number, Record<string, string>>;
  name: Record<string | number, string>;
}

Object.defineProperty(Chunk.prototype, 'getChunkMaps', {
  enumerable: true,
  configurable: true,
  value(this: Chunk, realHash: boolean): ChunkMaps {
    const chunkHashMap: Record<string | number, string> = {};
    const chunkContentHashMap: Record<
      string | number,
      Record<string, string>
    > = {};
    const chunkNameMap: Record<string | number, string> = {};

    for (const chunk of this.getAllAsyncChunks()) {
      const id = chunk.id;
      if (!id) continue;
      const chunkHash = realHash ? chunk.hash : chunk.renderedHash;
      if (chunkHash) {
        chunkHashMap[id] = chunkHash;
      }
      for (const key of Object.keys(chunk.contentHash)) {
        if (!chunkContentHashMap[key]) {
          chunkContentHashMap[key] = {};
        }
        chunkContentHashMap[key][id] = chunk.contentHash[key];
      }
      if (chunk.name) {
        chunkNameMap[id] = chunk.name;
      }
    }

    return {
      hash: chunkHashMap,
      contentHash: chunkContentHashMap,
      name: chunkNameMap,
    };
  },
});

Object.defineProperty(Chunk.prototype, util.inspect.custom, {
  enumerable: true,
  configurable: true,
  value(this: Chunk): any {
    return { ...this };
  },
});

declare module '@rspack/binding' {
  interface Chunk {
    readonly files: ReadonlySet<string>;
    readonly runtime: ReadonlySet<string>;
    readonly auxiliaryFiles: ReadonlySet<string>;
    readonly groupsIterable: ReadonlySet<ChunkGroup>;
    getChunkMaps(realHash: boolean): ChunkMaps;
  }
}

export { Chunk } from '@rspack/binding';
