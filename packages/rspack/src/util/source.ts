import type { JsSource, JsSourceWithLazyMap } from '@rspack/binding';
import { RawSource, type Source, SourceMapSource } from 'webpack-sources';

const sourceCacheSymbol = Symbol.for('rspack.originalSource');
const sourceMapOverrides = new WeakMap<
  SourceMapSource,
  { value: string | undefined }
>();
type SourceWithCache = SourceMapSource & {
  [sourceCacheSymbol]?: JsSourceWithLazyMap;
};

function replaceSourceMap(source: SourceWithCache, value: string | undefined) {
  if (
    !Object.getOwnPropertyDescriptor(source, '_sourceMapAsString')?.configurable
  ) {
    return false;
  }
  Object.defineProperty(source, '_sourceMapAsString', {
    value,
    writable: true,
    configurable: true,
    enumerable: true,
  });
  delete source[sourceCacheSymbol];
  return true;
}

function setSourceMap(this: SourceWithCache, value: string | undefined) {
  if (replaceSourceMap(this, value)) return;
  if (Object.isFrozen(this)) {
    throw new TypeError(
      "Cannot assign to read only property '_sourceMapAsString'",
    );
  }
  // A sealed source keeps the accessor; preserve its writable value for clearCache().
  sourceMapOverrides.set(this, { value });
}

function getSourceMap(this: SourceWithCache): string | undefined {
  const override = sourceMapOverrides.get(this);
  if (override) return override.value;
  const cache = this[sourceCacheSymbol];
  // Preserve extracted getters after the instance has materialized its map.
  if (!cache) return this._sourceMapAsString;
  const map = cache.map;
  const json = typeof map === 'string' ? map : map!();
  cache.map = json;
  // Frozen/sealed sources can still read the shared JSON without changing their descriptors.
  replaceSourceMap(this, json);
  return json;
}

export class SourceAdapter {
  static fromBinding(cache: JsSource | JsSourceWithLazyMap): Source {
    if (!cache.map) return new RawSource(cache.source);
    if (typeof cache.map === 'string') {
      return new SourceMapSource(
        cache.source,
        'inmemory://from rust',
        cache.map,
      );
    }
    const source = new SourceMapSource(cache.source, 'inmemory://from rust');
    source._hasSourceMap = true;
    Object.defineProperties(source, {
      [sourceCacheSymbol]: { value: cache, configurable: true },
      _sourceMapAsString: {
        get: getSourceMap,
        set: setSourceMap,
        configurable: true,
        enumerable: true,
      },
    });
    return source;
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
