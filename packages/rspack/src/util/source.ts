import type {
  JsNativeSource,
  JsSource,
  JsSourceFromJs,
  JsSourceWithLazyMap,
} from '@rspack/binding';
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

function getNativeSource(source: Source): JsNativeSource | undefined {
  const native = nativeSources.get(source);
  if (
    native &&
    source instanceof SourceMapSource &&
    (source._sourceMapAsObject !== undefined ||
      source._sourceMapAsBuffer !== undefined ||
      source._valueAsBuffer !== undefined)
  ) {
    // Exposed maps and buffers are mutable, so serialize the JS snapshot again.
    nativeSources.delete(source);
    return;
  }
  return native;
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
    const children = this.getChildren().map((child) =>
      SourceAdapter.toBinding(child),
    );
    // JS ConcatSource.source() coerces Buffers to text before serialization.
    if (
      children.some(
        (child) => child.kind === 'raw' && Buffer.isBuffer(child.content),
      )
    ) {
      return null;
    }
    return {
      kind: 'concat',
      children,
    };
  }
}

export class RspackReplaceSource extends webpackSources.ReplaceSource {
  [toBindingSource](): JsSourceFromJs | null {
    if (Object.getPrototypeOf(this) !== RspackReplaceSource.prototype) {
      return null;
    }
    const original = getNativeSource(this.original());
    if (!original) return null;
    const replaced = original.applyReplacements(this.getReplacements());
    return replaced ? { kind: 'native', handle: replaced } : null;
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

    // A Buffer returned by RawSource.source() can be changed without notifying us.
    if (nativeSource && typeof cache.source === 'string') {
      nativeSources.set(source, nativeSource);
    }
    return source;
  }

  static toBinding(source: Source): JsSourceFromJs {
    const nativeSource = getNativeSource(source);
    if (nativeSource) {
      return { kind: 'native', handle: nativeSource };
    }

    const structured = (source as SourceWithBinding)[toBindingSource]?.call(
      source,
    );
    if (structured) return structured;

    const content = source.source();
    if (Buffer.isBuffer(content)) {
      return {
        kind: 'raw',
        content,
      };
    }

    const map = source.map?.({
      columns: true,
    });
    return {
      kind: 'raw',
      content,
      map: map ? JSON.stringify(map) : undefined,
    };
  }
}
