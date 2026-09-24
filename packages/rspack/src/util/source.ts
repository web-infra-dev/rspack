import type { JsSource, JsSourceLazy } from '@rspack/binding';
import {
  type GeneratedSourceInfo,
  type HashLike,
  type MapOptions,
  RawSource,
  type RawSourceMap,
  Source,
  type SourceAndMap,
  SourceMapSource,
  type StreamChunksOptions,
} from 'webpack-sources';

type MaterializedSource = RawSource | SourceMapSource;

function isLazySource(source: JsSource | JsSourceLazy): source is JsSourceLazy {
  return (source as JsSourceLazy).lazy === true;
}

/**
 * A `Source` over a lazy binding source.
 *
 * Reading `originalSource` usually only needs the source text, while the source map can be much
 * larger than the source itself. Building the `SourceMapSource` eagerly for every module keeps
 * the raw map JSON string alive in the JS heap, so both the text and the map are only converted
 * when an API that needs them is used.
 */
class LazySource extends Source {
  #binding: JsSourceLazy;
  #value: string | Buffer | undefined;
  #materialized: MaterializedSource | undefined;

  constructor(binding: JsSourceLazy) {
    super();
    this.#binding = binding;
  }

  #sourceValue(): string | Buffer {
    if (this.#value === undefined) {
      this.#value = this.#binding.source;
    }
    return this.#value;
  }

  /**
   * The eager source this lazy source stands for. The binding source map is only read here, which
   * is what `SourceAdapter.fromBinding` would have done up front for an eager source.
   */
  #delegate(): MaterializedSource {
    if (this.#materialized === undefined) {
      const value = this.#sourceValue();
      if (Buffer.isBuffer(value)) {
        this.#materialized = new RawSource(value);
      } else {
        const map = this.#binding.map;
        this.#materialized =
          map === undefined
            ? new RawSource(value)
            : new SourceMapSource(value, 'inmemory://from rust', map);
      }
    }
    return this.#materialized;
  }

  source(): string | Buffer {
    return this.#sourceValue();
  }

  buffer(): Buffer {
    const value = this.#sourceValue();
    return Buffer.isBuffer(value) ? value : Buffer.from(value, 'utf8');
  }

  size(): number {
    const value = this.#sourceValue();
    return Buffer.isBuffer(value)
      ? value.length
      : Buffer.byteLength(value, 'utf8');
  }

  map(options?: MapOptions): null | RawSourceMap {
    return this.#delegate().map(options) ?? null;
  }

  sourceAndMap(options?: MapOptions): SourceAndMap {
    return this.#delegate().sourceAndMap(options);
  }

  streamChunks(
    options: StreamChunksOptions,
    onChunk: (
      chunk: undefined | string,
      generatedLine: number,
      generatedColumn: number,
      sourceIndex: number,
      originalLine: number,
      originalColumn: number,
      nameIndex: number,
    ) => void,
    onSource: (
      sourceIndex: number,
      source: null | string,
      sourceContent?: string,
    ) => void,
    onName: (nameIndex: number, name: string) => void,
  ): GeneratedSourceInfo {
    return this.#delegate().streamChunks(options, onChunk, onSource, onName);
  }

  updateHash(hash: HashLike): void {
    this.#delegate().updateHash(hash);
  }

  clearCache(
    options?: { source?: boolean; maps?: boolean; parsedMap?: boolean },
    visited?: WeakSet<Source>,
  ): void {
    if (visited !== undefined) {
      if (visited.has(this)) {
        return;
      }
      visited.add(this);
    }
    if (!options || options.source !== false) {
      this.#value = undefined;
    }
    if (!options || options.maps !== false) {
      this.#materialized = undefined;
    }
  }
}

export class SourceAdapter {
  static fromBinding(source: JsSource | JsSourceLazy): Source {
    // Lazy binding sources know how to materialize their text and map on demand.
    if (isLazySource(source)) {
      return new LazySource(source);
    }
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
