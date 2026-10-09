import type { JsSource, JsSourceWithLazyMap, Module } from '@rspack/binding';
import { RawSource, type Source, SourceMapSource } from 'webpack-sources';

const sourceCacheSymbol = Symbol.for('rspack.originalSource');
type ModuleWithSourceCache = Module & {
  [sourceCacheSymbol]?: JsSourceWithLazyMap;
};
type SourceWithCache = SourceMapSource & {
  [sourceCacheSymbol]?: JsSourceWithLazyMap;
};

function getSourceMap(this: SourceWithCache): string | undefined {
  const cache = this[sourceCacheSymbol];
  // Preserve extracted getters after the instance has materialized its map.
  if (!cache) return this._sourceMapAsString;
  const map = cache.map;
  const json = typeof map === 'string' ? map : map!.takeJson();
  cache.map = json;
  Object.defineProperty(this, '_sourceMapAsString', {
    value: json,
    writable: true,
    configurable: true,
    enumerable: true,
  });
  delete this[sourceCacheSymbol];
  return json;
}

export class SourceAdapter {
  static fromModule(module: ModuleWithSourceCache): Source | null {
    let cache = module[sourceCacheSymbol];
    if (!cache || !module._isOriginalSource(cache)) {
      cache = module._originalSource();
      if (!cache) {
        delete module[sourceCacheSymbol];
        return null;
      }
      Object.defineProperty(module, sourceCacheSymbol, {
        value: cache,
        writable: true,
        configurable: true,
      });
    }
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
        configurable: true,
        enumerable: true,
      },
    });
    return source;
  }

  static fromBinding(source: JsSource): Source {
    const { source: content, map } = source;
    if (!map) {
      return new RawSource(content);
    }
    return new SourceMapSource(content, 'inmemory://from rust', map);
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
