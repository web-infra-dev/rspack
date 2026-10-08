import { createHash } from 'node:crypto';
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
          compiler.hooks.compilation.tap(
            'OriginalSourcePlugin',
            (compilation) => {
              compilation.hooks.succeedModule.tap(
                'OriginalSourcePlugin',
                (module) => {
                  const source = module.originalSource();
                  if (!source) return;

                  const value = source.source();
                  const binding = module._originalSource()!;
                  const map = binding.map?.toJson();
                  if (Buffer.isBuffer(value)) {
                    checkedBuffer = true;
                    expect(value).toEqual(Buffer.from([0, 255, 97]));
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

                  if (!binding.map) return;
                  let transfers = 0;
                  // Observe the JSON transfer boundary without inspecting the map proxy.
                  const createSource = () =>
                    module.originalSource.call({
                      _originalSource: () => ({
                        source: binding.source,
                        map: {
                          toJson() {
                            transfers++;
                            return binding.map!.toJson();
                          },
                        },
                      }),
                    })!;
                  const lazy = createSource();
                  expect(lazy).toBeInstanceOf(sources.SourceMapSource);
                  expect(lazy.source()).toEqual(value);
                  expect(lazy.buffer()).toEqual(eager.buffer());
                  expect(lazy.size()).toBe(eager.size());
                  const lazyMap = lazy.map()!;
                  expect(lazy.map()).toBe(lazyMap);
                  expect(lazy.map({ columns: false })).toBe(lazyMap);
                  expect(lazy.sourceAndMap().map).toBe(lazyMap);
                  expect(transfers).toBe(0);
                  expect(lazyMap.mappings).toBe(eager.map()!.mappings);
                  expect(transfers).toBe(1);
                  expect(lazy.map()).toBe(lazyMap);
                  expect(lazy.sourceAndMap().map).toBe(lazyMap);
                  expect(JSON.stringify(lazyMap)).toBe(map);
                  expect(transfers).toBe(1);

                  const operations: ((map: RawSourceMap) => unknown)[] = [
                    (map) => map.sourcesContent,
                    (map) => JSON.stringify(map),
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
                    const actual = createSource().map()!;
                    const expected = JSON.parse(map!);
                    expect(transfers).toBe(previousTransfers);
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
                },
              );
            },
          );
          compiler.hooks.done.tap('OriginalSourcePlugin', () => {
            expect(checkedText).toBe(true);
            expect(checkedBuffer).toBe(true);
          });
        },
      },
    ],
  }),
);
