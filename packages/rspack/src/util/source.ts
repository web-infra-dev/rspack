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
  [toBindingSource](): JsSourceFromJs {
    return {
      kind: 'concat',
      children: this.getChildren().map((child) =>
        SourceAdapter.toBinding(child),
      ),
    };
  }
}

function splitsSurrogatePair(text: string, position: number) {
  if (position <= 0 || position >= text.length) return false;
  const previous = text.charCodeAt(position - 1);
  const next = text.charCodeAt(position);
  return (
    previous >= 0xd800 && previous <= 0xdbff && next >= 0xdc00 && next <= 0xdfff
  );
}

export class RspackReplaceSource extends webpackSources.ReplaceSource {
  [toBindingSource](): JsSourceFromJs | null {
    const original = this.original();
    const replacements = this.getReplacements();
    if (replacements.length === 0) {
      return SourceAdapter.toBinding(original);
    }

    const text = original.source();
    if (typeof text !== 'string') return null;

    const edits: Array<{ start: number; end: number; content: string }> = [];
    let cursor = 0;
    let bytes = 0;
    for (const replacement of replacements) {
      const start = Math.floor(replacement.start);
      const end = Math.floor(replacement.end + 1);
      if (
        !Number.isSafeInteger(start) ||
        !Number.isSafeInteger(end) ||
        start < cursor ||
        end < start ||
        end > text.length ||
        replacement.name ||
        splitsSurrogatePair(text, start) ||
        splitsSurrogatePair(text, end)
      ) {
        return null;
      }

      bytes += Buffer.byteLength(text.slice(cursor, start));
      const byteStart = bytes;
      bytes += Buffer.byteLength(text.slice(start, end));
      if (bytes > 0xffffffff) return null;
      cursor = end;
      edits.push({
        start: byteStart,
        end: bytes,
        content: replacement.content,
      });
    }

    return {
      kind: 'replace',
      original: SourceAdapter.toBinding(original),
      replacements: edits,
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
    const nativeSource = nativeSources.get(source);
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
