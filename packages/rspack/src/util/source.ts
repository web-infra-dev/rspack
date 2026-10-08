import type {
  JsSource,
  JsSourceMap,
  JsSourceWithLazyMap,
} from '@rspack/binding';
import {
  RawSource,
  type RawSourceMap,
  type Source,
  SourceMapSource,
} from 'webpack-sources';

function createLazySourceMap(binding: JsSourceMap): RawSourceMap {
  let pending: JsSourceMap | undefined = binding;
  const materialize = (target: RawSourceMap): RawSourceMap => {
    if (pending !== undefined) {
      // Populate the proxy target itself so descriptors, mutations and Object.freeze obey
      // ordinary object semantics, including the proxy invariants for non-configurable keys.
      Object.defineProperties(
        target,
        Object.getOwnPropertyDescriptors(JSON.parse(pending.toJson())),
      );
      pending = undefined;
    }
    return target;
  };

  return new Proxy({} as RawSourceMap, {
    get(target, property, receiver) {
      return Reflect.get(materialize(target), property, receiver);
    },
    set(target, property, value, receiver) {
      return Reflect.set(materialize(target), property, value, receiver);
    },
    has(target, property) {
      return Reflect.has(materialize(target), property);
    },
    ownKeys(target) {
      return Reflect.ownKeys(materialize(target));
    },
    getOwnPropertyDescriptor(target, property) {
      return Reflect.getOwnPropertyDescriptor(materialize(target), property);
    },
    defineProperty(target, property, descriptor) {
      return Reflect.defineProperty(materialize(target), property, descriptor);
    },
    deleteProperty(target, property) {
      return Reflect.deleteProperty(materialize(target), property);
    },
    preventExtensions(target) {
      return Reflect.preventExtensions(materialize(target));
    },
  });
}

export class SourceAdapter {
  static fromBinding(source: JsSource | JsSourceWithLazyMap): Source {
    const map = source.map;
    if (!map) {
      return new RawSource(source.source);
    }
    return new SourceMapSource(
      source.source,
      'inmemory://from rust',
      typeof map === 'string' ? map : createLazySourceMap(map),
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
