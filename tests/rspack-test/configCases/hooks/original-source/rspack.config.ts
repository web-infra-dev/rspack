import { createHash } from 'node:crypto';
import { isProxy } from 'node:util/types';
import { defineConfig } from '@rspack/cli';
import { type Compiler, sources } from '@rspack/core';
import type { RawSourceMap, Source } from 'webpack-sources';

function hashSource(source: Source) {
  const hash = createHash('sha256');
  source.updateHash(hash);
  return hash.digest('hex');
}

export default ([false, 'source-map'] as const).map((devtool) =>
  defineConfig({
    devtool,
    module: {
      rules: [{ mimetype: 'application/octet-stream', type: 'asset/inline' }],
    },
    plugins: [
      {
        apply(compiler: Compiler) {
          let checkedText = false;
          let checkedBuffer = false;
          let checkedEmptyBuffer = false;
          compiler.hooks.compilation.tap(
            'OriginalSourcePlugin',
            (compilation) => {
              compilation.hooks.succeedModule.tap(
                'OriginalSourcePlugin',
                (module) => {
                  const source = module.originalSource();
                  if (!source) return;

                  const value = source.source();
                  const binding = module._originalSource(
                    sources.SourceMapSource,
                  )!;
                  let map: string | undefined;
                  let nativeGetter: (() => string) | undefined;
                  if (binding instanceof sources.SourceMapSource) {
                    // The binding already returned the actual SourceMapSource, with one shared
                    // native accessor and no intermediate JsSourceMap object or JS getter closure.
                    nativeGetter = Object.getOwnPropertyDescriptor(
                      binding,
                      '_sourceMapAsString',
                    )!.get;
                    expect(nativeGetter).toBeDefined();
                    expect(
                      Function.prototype.toString.call(nativeGetter),
                    ).toContain('[native code]');
                    expect(binding._sourceMapAsObject).toBeUndefined();
                    expect(binding._sourceMapAsBuffer).toBeUndefined();
                    expect(
                      module.originalSource.call({
                        _originalSource: () => binding,
                      }),
                    ).toBe(binding);
                    map = binding._sourceMapAsString;
                  }
                  if (Buffer.isBuffer(value)) {
                    if (module.identifier().endsWith('base64,')) {
                      checkedEmptyBuffer = true;
                      expect(value).toEqual(Buffer.alloc(0));
                    } else {
                      checkedBuffer = true;
                      expect(value).toEqual(Buffer.from([0, 255, 97]));
                    }
                    expect(map).toBeUndefined();
                  } else {
                    checkedText = true;
                    expect(value).toContain('你好 🌍');
                    if (devtool) {
                      expect(typeof map).toBe('string');
                    } else {
                      expect(map).toBeUndefined();
                    }
                  }

                  const eager = map
                    ? new sources.SourceMapSource(
                        value,
                        'inmemory://from rust',
                        map,
                      )
                    : new sources.RawSource(value);
                  expect(Object.getPrototypeOf(source)).toBe(
                    Object.getPrototypeOf(eager),
                  );
                  expect(isProxy(source)).toBe(false);
                  expect(source.map).toBe(eager.map);
                  expect(source.updateHash).toBe(eager.updateHash);
                  if (source instanceof sources.RawSource) {
                    expect(source.isBuffer()).toBe(Buffer.isBuffer(value));
                    expect(source.map()).toBeNull();
                  } else {
                    expect(source).toBeInstanceOf(sources.SourceMapSource);
                    expect(
                      (source as sources.SourceMapSource).getArgsAsBuffers(),
                    ).toEqual(
                      (eager as sources.SourceMapSource).getArgsAsBuffers(),
                    );
                  }
                  for (const options of [
                    undefined,
                    { maps: false },
                    { source: false },
                    { parsedMap: true },
                  ]) {
                    source.clearCache(options);
                    eager.clearCache(options);
                    expect(source.source()).toEqual(eager.source());
                    expect(source.size()).toBe(eager.size());
                    expect(source.buffer()).toEqual(eager.buffer());
                    expect(source.buffers()).toEqual(eager.buffers());
                    expect(source.sourceAndMap()).toEqual(eager.sourceAndMap());
                    expect(hashSource(source)).toBe(hashSource(eager));
                    for (const columns of [true, false]) {
                      expect(
                        new sources.ConcatSource(source).sourceAndMap({
                          columns,
                        }),
                      ).toEqual(
                        new sources.ConcatSource(eager).sourceAndMap({
                          columns,
                        }),
                      );
                    }
                  }

                  if (Buffer.isBuffer(value) && value.length > 0) {
                    // The emitted data URL must still contain the original Rust source bytes.
                    value[0] = 42;
                    expect(source.buffer()[0]).toBe(42);
                  }

                  if (!map) return;
                  let transfers = 0;
                  // Count calls to the real native getter without reading its map data.
                  const createSource = () => {
                    const result = module.originalSource()!;
                    const descriptor = Object.getOwnPropertyDescriptor(
                      result,
                      '_sourceMapAsString',
                    )!;
                    expect(descriptor.get).toBe(nativeGetter);
                    Object.defineProperty(result, '_sourceMapAsString', {
                      ...descriptor,
                      get() {
                        transfers++;
                        return descriptor.get!.call(this);
                      },
                    });
                    return result;
                  };
                  const lazy = createSource();
                  expect(lazy).toBeInstanceOf(sources.SourceMapSource);
                  expect(lazy.source()).toEqual(value);
                  expect(lazy.buffer()).toEqual(eager.buffer());
                  expect(lazy.size()).toBe(eager.size());
                  expect(transfers).toBe(0);
                  const lazyMap = lazy.map()!;
                  expect(transfers).toBe(1);
                  expect(isProxy(lazyMap)).toBe(false);
                  expect(Object.getPrototypeOf(lazyMap)).toBe(Object.prototype);
                  expect(lazy.map()).toBe(lazyMap);
                  expect(lazy.map({ columns: false })).toBe(lazyMap);
                  expect(lazy.sourceAndMap().map).toBe(lazyMap);
                  expect(transfers).toBe(1);
                  expect(lazyMap.mappings).toBe(eager.map()!.mappings);
                  expect(transfers).toBe(1);
                  expect(lazy.map()).toBe(lazyMap);
                  expect(lazy.sourceAndMap().map).toBe(lazyMap);
                  expect(JSON.stringify(lazyMap)).toBe(map);
                  expect(transfers).toBe(1);

                  const operations: ((map: RawSourceMap) => unknown)[] = [
                    (map) => map.sourcesContent,
                    (map) => JSON.stringify(map),
                    (map) => structuredClone(map),
                    (map) => Object.keys(map),
                    (map) => ({ ...map }),
                    (map) => 'mappings' in map,
                    (map) => Object.getOwnPropertyDescriptor(map, 'mappings'),
                    (map) => {
                      map.file = 'changed.js';
                      return map.file;
                    },
                    (map) =>
                      Object.defineProperty(map, 'file', {
                        value: 'defined.js',
                      }),
                    (map) => Reflect.deleteProperty(map, 'sourcesContent'),
                    (map) => Object.freeze(map),
                    (map) => Object.seal(map),
                    (map) => Object.preventExtensions(map),
                  ];
                  for (const operation of operations) {
                    const previousTransfers = transfers;
                    const actualSource = createSource();
                    expect(transfers).toBe(previousTransfers);
                    const actual = actualSource.map()!;
                    const expected = JSON.parse(map!);
                    expect(transfers).toBe(previousTransfers + 1);
                    expect(operation(actual)).toEqual(operation(expected));
                    expect(actual).toEqual(expected);
                    expect(Object.getOwnPropertyDescriptors(actual)).toEqual(
                      Object.getOwnPropertyDescriptors(expected),
                    );
                    expect(Object.isExtensible(actual)).toBe(
                      Object.isExtensible(expected),
                    );
                    expect(transfers).toBe(previousTransfers + 1);
                  }
                  // Each public Source has its own mutable map, as with the eager JSON input.
                  const secondMap = createSource().map()!;
                  secondMap.sources.push('added.js');
                  expect(lazyMap.sources).not.toContain('added.js');
                  expect(transfers).toBe(operations.length + 2);

                  const caching = sources.util.stringBufferUtils;
                  const wasCaching = caching.isDualStringBufferCachingEnabled();
                  try {
                    for (const cacheBuffers of [true, false]) {
                      if (cacheBuffers) caching.enableDualStringBufferCaching();
                      else caching.disableDualStringBufferCaching();
                      for (const readMapFirst of [false, true]) {
                        const actual =
                          createSource() as sources.SourceMapSource;
                        const reference = new sources.SourceMapSource(
                          value,
                          'inmemory://from rust',
                          map!,
                        );
                        const previousTransfers = transfers;
                        if (readMapFirst) {
                          actual.sourceAndMap().map!.sources[0] = 'changed.js';
                          reference.sourceAndMap().map!.sources[0] =
                            'changed.js';
                          expect(transfers).toBe(previousTransfers + 1);
                        }
                        expect(hashSource(actual)).toBe(hashSource(reference));
                        expect(actual.getArgsAsBuffers()).toEqual(
                          reference.getArgsAsBuffers(),
                        );
                        expect(hashSource(actual)).toBe(hashSource(reference));
                        expect(actual.map()).toEqual(reference.map());
                        expect(transfers).toBe(previousTransfers + 1);
                        for (const options of [
                          undefined,
                          { maps: false },
                          { parsedMap: true },
                        ]) {
                          actual.clearCache(options);
                          reference.clearCache(options);
                          expect(hashSource(actual)).toBe(
                            hashSource(reference),
                          );
                          expect(actual.map()).toEqual(reference.map());
                        }
                        expect(transfers).toBe(previousTransfers + 1);
                      }
                    }
                  } finally {
                    if (wasCaching) caching.enableDualStringBufferCaching();
                    else caching.disableDualStringBufferCaching();
                  }
                },
              );
            },
          );
          compiler.hooks.done.tap('OriginalSourcePlugin', () => {
            expect(checkedText).toBe(true);
            expect(checkedBuffer).toBe(true);
            expect(checkedEmptyBuffer).toBe(true);
          });
        },
      },
    ],
  }),
);
