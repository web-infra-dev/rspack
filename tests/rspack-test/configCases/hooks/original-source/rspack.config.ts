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
          const {
            Module,
            NormalModule,
            ConcatenatedModule,
            ContextModule,
            ExternalModule,
          } = compiler.rspack;
          for (const constructor of [
            Module,
            NormalModule,
            ConcatenatedModule,
            ContextModule,
            ExternalModule,
          ]) {
            expect('_isOriginalSource' in constructor.prototype).toBe(false);
          }
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
                  const binding = module.originalSource()!;
                  let map: string | undefined;
                  let mapGetter: (() => string) | undefined;
                  if (binding instanceof sources.SourceMapSource) {
                    const cacheSymbol = Symbol.for('rspack.originalSource');
                    const cache = Reflect.get(source, cacheSymbol);
                    expect(Reflect.has(module, cacheSymbol)).toBe(false);
                    expect(Reflect.get(binding, cacheSymbol)).toBe(cache);
                    expect(Reflect.get(source, cacheSymbol)).toBe(cache);
                    const nativeMap = cache.map;
                    expect(typeof nativeMap.takeJson).toBe('function');
                    // JavaScript constructs the real class and shares one getter across instances.
                    mapGetter = Object.getOwnPropertyDescriptor(
                      binding,
                      '_sourceMapAsString',
                    )!.get;
                    expect(typeof mapGetter).toBe('function');
                    expect(binding._sourceMapAsObject).toBeUndefined();
                    expect(binding._sourceMapAsBuffer).toBeUndefined();
                    expect(source.buffer()).toEqual(Buffer.from(value));
                    expect(source.size()).toBe(Buffer.byteLength(value));
                    expect(
                      Object.getOwnPropertyDescriptor(
                        source,
                        '_sourceMapAsString',
                      )!.get,
                    ).toBe(mapGetter);
                    expect(module.originalSource()).not.toBe(source);
                    map = binding._sourceMapAsString;
                    expect(cache.map).toBe(map);
                    expect(() => nativeMap.takeJson()).toThrow(
                      'Source map has already been consumed',
                    );
                    expect(Reflect.has(binding, cacheSymbol)).toBe(false);
                    expect(
                      Object.getOwnPropertyDescriptor(
                        binding,
                        '_sourceMapAsString',
                      ),
                    ).toEqual({
                      value: map,
                      writable: true,
                      enumerable: true,
                      configurable: true,
                    });
                    // An instance created before the transfer reuses the shared JS JSON.
                    expect(mapGetter!.call(source)).toBe(map);
                    expect(Reflect.has(source, cacheSymbol)).toBe(false);
                    expect(
                      Object.getOwnPropertyDescriptor(
                        source,
                        '_sourceMapAsString',
                      )!.get,
                    ).toBeUndefined();
                    expect(mapGetter!.call(source)).toBe(map);
                    // Instances created afterwards receive that JSON directly in their constructor.
                    const initialized =
                      module.originalSource() as sources.SourceMapSource;
                    expect(initialized._sourceMapAsString).toBe(map);
                    expect(Reflect.has(initialized, cacheSymbol)).toBe(false);
                    expect(
                      Object.getOwnPropertyDescriptor(
                        initialized,
                        '_sourceMapAsString',
                      )!.get,
                    ).toBeUndefined();
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
                  // Later instances have ordinary data properties and no map getter.
                  const createSource = () => {
                    const result = module.originalSource()!;
                    const descriptor = Object.getOwnPropertyDescriptor(
                      result,
                      '_sourceMapAsString',
                    )!;
                    expect(descriptor.get).toBeUndefined();
                    expect(descriptor.value).toBe(map);
                    return result;
                  };
                  const lazy = createSource();
                  expect(lazy).toBeInstanceOf(sources.SourceMapSource);
                  expect(lazy.source()).toEqual(value);
                  expect(lazy.buffer()).toEqual(eager.buffer());
                  expect(lazy.size()).toBe(eager.size());
                  const lazyMap = lazy.map()!;
                  expect(isProxy(lazyMap)).toBe(false);
                  expect(Object.getPrototypeOf(lazyMap)).toBe(Object.prototype);
                  expect(lazy.map()).toBe(lazyMap);
                  expect(lazy.map({ columns: false })).toBe(lazyMap);
                  expect(lazy.sourceAndMap().map).toBe(lazyMap);
                  expect(lazyMap.mappings).toBe(eager.map()!.mappings);
                  expect(lazy.map()).toBe(lazyMap);
                  expect(lazy.sourceAndMap().map).toBe(lazyMap);
                  expect(JSON.stringify(lazyMap)).toBe(map);

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
                    const actualSource = createSource();
                    const actual = actualSource.map()!;
                    const expected = JSON.parse(map!);
                    expect(operation(actual)).toEqual(operation(expected));
                    expect(actual).toEqual(expected);
                    expect(Object.getOwnPropertyDescriptors(actual)).toEqual(
                      Object.getOwnPropertyDescriptors(expected),
                    );
                    expect(Object.isExtensible(actual)).toBe(
                      Object.isExtensible(expected),
                    );
                  }
                  // Each public Source has its own mutable map, as with the eager JSON input.
                  const secondMap = createSource().map()!;
                  secondMap.sources.push('added.js');
                  expect(lazyMap.sources).not.toContain('added.js');

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
                        if (readMapFirst) {
                          actual.sourceAndMap().map!.sources[0] = 'changed.js';
                          reference.sourceAndMap().map!.sources[0] =
                            'changed.js';
                        }
                        expect(hashSource(actual)).toBe(hashSource(reference));
                        expect(actual.getArgsAsBuffers()).toEqual(
                          reference.getArgsAsBuffers(),
                        );
                        expect(hashSource(actual)).toBe(hashSource(reference));
                        expect(actual.map()).toEqual(reference.map());
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
