import type { JsSource } from '@rspack/binding';
import { RawSource, type Source, SourceMapSource } from 'webpack-sources';

export class SourceAdapter {
  static fromBinding(source: JsSource | SourceMapSource): Source {
    if (source instanceof SourceMapSource) return source;
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
