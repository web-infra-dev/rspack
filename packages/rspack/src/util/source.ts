import type {
  JsNativeSource,
  JsSource,
  JsSourceFromJs,
  JsSourceWithLazyMap,
} from '@rspack/binding';
import binding from '@rspack/binding';
import * as webpackSources from 'webpack-sources';
import { RawSource, type Source, SourceMapSource } from 'webpack-sources';

const sourceCacheSymbol = Symbol.for('rspack.originalSource');
const nativeSources = new WeakMap<Source, JsNativeSource>();
const toBindingSource = Symbol('rspack.toBindingSource');
const sourceMapOverrides = new WeakMap<
  SourceMapSource,
  { value: string | undefined }
>();
type SourceWithCache = SourceMapSource & {
  [sourceCacheSymbol]?: JsSourceWithLazyMap;
};
type SourceWithBinding = Source & {
  [toBindingSource]?: () => JsSourceFromJs | null;
};

function toSourceGraph(source: Source): JsSourceFromJs {
  const native = nativeSources.get(source);
  if (native) return { kind: 'native', handle: native };
  return (
    (source as SourceWithBinding)[toBindingSource]?.call(source) ??
    serializeSource(source)
  );
}

function serializeSource(source: Source): JsSourceFromJs {
  const content = source.source();
  if (Buffer.isBuffer(content)) return { kind: 'raw', content };
  const map = source.map?.({ columns: true });
  return { kind: 'raw', content, map: map ? JSON.stringify(map) : undefined };
}

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

function clearSourceCache(
  this: SourceWithCache,
  options?: Source.ClearCacheOptions,
  visited?: WeakSet<Source>,
) {
  // An unread native map has no JS map caches to release. Avoid probing its getter.
  if (typeof this[sourceCacheSymbol]?.map === 'function') {
    options = { ...options, maps: false };
  }
  SourceMapSource.prototype.clearCache.call(this, options, visited);
}

export class RspackConcatSource extends webpackSources.ConcatSource {
  [toBindingSource](): JsSourceFromJs | null {
    if (Object.getPrototypeOf(this) !== RspackConcatSource.prototype) {
      return null;
    }
    return {
      kind: 'concat',
      children: this.getChildren().map(toSourceGraph),
    };
  }
}

export class RspackReplaceSource extends webpackSources.ReplaceSource {
  [toBindingSource](): JsSourceFromJs | null {
    if (Object.getPrototypeOf(this) !== RspackReplaceSource.prototype) {
      return null;
    }
    return {
      kind: 'replace',
      original: toSourceGraph(this.original()),
      replacements: this.getReplacements(),
      fallback: () => serializeSource(this),
    };
  }
}

export class SourceAdapter {
  static fromBinding(
    cache: JsSource | JsSourceWithLazyMap,
    nativeSource?: JsNativeSource | null,
  ): Source {
    let source: Source;
    if (!cache.map) {
      source = new RawSource(cache.source);
    } else if (typeof cache.map === 'string') {
      source = new SourceMapSource(
        cache.source,
        'inmemory://from rust',
        cache.map,
      );
    } else {
      const sourceMapSource = new SourceMapSource(
        cache.source,
        'inmemory://from rust',
      );
      sourceMapSource._hasSourceMap = true;
      Object.defineProperties(sourceMapSource, {
        [sourceCacheSymbol]: { value: cache, configurable: true },
        clearCache: {
          value: clearSourceCache,
          writable: true,
          configurable: true,
        },
        _sourceMapAsString: {
          get: getSourceMap,
          set: setSourceMap,
          configurable: true,
          enumerable: true,
        },
      });
      source = sourceMapSource;
    }

    if (nativeSource) {
      nativeSources.set(source, nativeSource);
    }
    return source;
  }

  static toBinding(source: Source): JsSourceFromJs {
    const graph = toSourceGraph(source);
    if (graph.kind === 'raw' || graph.kind === 'native') return graph;
    return { kind: 'native', handle: binding.JsNativeSource.fromSource(graph) };
  }
}
