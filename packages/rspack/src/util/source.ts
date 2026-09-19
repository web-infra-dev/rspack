import type { JsSource, JsSourceSnapshot } from '@rspack/binding';
import { RawSource, Source, SourceMapSource } from 'webpack-sources';

class NativeSource extends Source {
  #state:
    | { snapshot: JsSourceSnapshot; content?: string | Buffer }
    | { source: RawSource | SourceMapSource };

  constructor(snapshot: JsSourceSnapshot) {
    super();
    this.#state = { snapshot };
  }

  source(): string | Buffer {
    const state = this.#state;
    return 'source' in state
      ? state.source.source()
      : (state.content ??= state.snapshot.source());
  }

  buffer(): Buffer {
    // The adapter owns the mutable buffer, including its hash semantics.
    return this.#materialize().buffer();
  }

  size(): number {
    if ('source' in this.#state) {
      return this.#state.source.size();
    }
    const source = this.source();
    return Buffer.isBuffer(source) ? source.length : Buffer.byteLength(source);
  }

  #materialize(): RawSource | SourceMapSource {
    const state = this.#state;
    if ('source' in state) {
      return state.source;
    }
    // A returned Buffer belongs to this wrapper and may have been mutated.
    const source = Buffer.isBuffer(state.content)
      ? new RawSource(state.content)
      : SourceAdapter.fromBinding(state.snapshot.sourceAndMap());
    // The ordinary adapter now owns the content; release the native snapshot.
    this.#state = { source };
    return source;
  }

  map(options: Parameters<Source['map']>[0]) {
    return this.#materialize().map(options);
  }

  sourceAndMap(options: Parameters<Source['sourceAndMap']>[0]) {
    return this.#materialize().sourceAndMap(options);
  }

  streamChunks(...args: Parameters<SourceMapSource['streamChunks']>) {
    return this.#materialize().streamChunks(...args);
  }

  updateHash(hash: Parameters<Source['updateHash']>[0]): void {
    this.#materialize().updateHash(hash);
  }
}

export class SourceAdapter {
  static fromSnapshot(snapshot: JsSourceSnapshot): Source {
    return new NativeSource(snapshot);
  }

  static fromBinding(source: JsSource): RawSource | SourceMapSource {
    if (!source.map) {
      return new RawSource(source.source);
    }
    return new SourceMapSource(
      source.source,
      'inmemory://from rust',
      // see: https://github.com/webpack/webpack-sources/blob/9f98066311d53a153fdc7c633422a1d086528027/lib/SourceMapSource.js#L30
      source.map,
    );
  }

  static toBinding(source: Source): JsSource {
    const content = source.source();
    if (Buffer.isBuffer(content)) {
      return {
        source: content,
        map: undefined,
      };
    }

    const map = source.map?.({
      columns: true,
    });
    const stringifyMap = map ? JSON.stringify(map) : undefined;
    return {
      source: content,
      map: stringifyMap,
    };
  }
}
