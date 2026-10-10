import type { JsAssetSource, JsSource } from '@rspack/binding';
import {
  RawSource,
  SizeOnlySource,
  type Source,
  SourceMapSource,
} from 'webpack-sources';

export class SourceAdapter {
  static fromBinding(source: JsAssetSource): Source {
    if (typeof source.source === 'number') {
      return new SizeOnlySource(source.source);
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
