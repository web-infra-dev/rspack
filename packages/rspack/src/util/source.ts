import type { JsSource, JsSourceWithLazyMap } from '@rspack/binding';
import { RawSource, type Source, SourceMapSource } from 'webpack-sources';

export class SourceAdapter {
  static fromBinding(source: JsSource | JsSourceWithLazyMap): Source {
    const map = source.map;
    if (!map) {
      return new RawSource(source.source);
    }
    const result = new SourceMapSource(
      source.source,
      'inmemory://from rust',
      typeof map === 'string' ? map : undefined,
    );
    if (typeof map !== 'string') {
      result._hasSourceMap = true;
      // Start with the same JSON representation as an eager SourceMapSource, but obtain it
      // from N-API only when webpack-sources reads it. Afterwards its normal caches apply.
      Object.defineProperty(result, '_sourceMapAsString', {
        configurable: true,
        enumerable: true,
        get() {
          const json = map.toJson();
          Object.defineProperty(this, '_sourceMapAsString', {
            configurable: true,
            enumerable: true,
            writable: true,
            value: json,
          });
          return json;
        },
      });
    }
    return result;
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
