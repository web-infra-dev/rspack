const { createHash } = require('node:crypto');
const { createRequire } = require('node:module');
const { JsSourceSnapshot } = createRequire(require.resolve('@rspack/core'))(
  '@rspack/binding',
);

function digest(source) {
  const hash = createHash('sha256');
  source.updateHash(hash);
  return hash.digest('hex');
}

function stream(source, options) {
  const events = [];
  const result = source.streamChunks(
    options || {},
    (...args) => events.push(['chunk', ...args]),
    (...args) => events.push(['source', ...args]),
    (...args) => events.push(['name', ...args]),
  );
  return { events, result };
}

module.exports = {
  description: 'retains source snapshots, maps and mutable buffers after close',
  options() {
    const retained = [];
    this.retained = retained;
    return {
      plugins: [
        {
          apply(compiler) {
            const { RawSource, OriginalSource, SourceMapSource } =
              compiler.rspack.sources;
            compiler.hooks.compilation.tap('SourceSnapshots', (compilation) => {
              compilation.hooks.processAssets.tap('SourceSnapshots', () => {
                expect(compilation.getAssetSource('missing')).toBeUndefined();
                compilation.emitAsset(
                  'lazy.js',
                  new OriginalSource('globalThis.lazy = 1;\n', 'lazy.ts'),
                );
                const materialize = rstest.spyOn(
                  JsSourceSnapshot.prototype,
                  'sourceAndMap',
                );
                try {
                  const lazy = compilation.getAsset('lazy.js').source;
                  expect(lazy.source()).toBe('globalThis.lazy = 1;\n');
                  expect(lazy.size()).toBe(21);
                  expect(materialize).not.toHaveBeenCalled();
                  expect(lazy.map().sources).toEqual(['lazy.ts']);
                  expect(materialize).toHaveBeenCalledTimes(1);
                  lazy.sourceAndMap();
                  lazy.updateHash(createHash('sha256'));
                  expect(materialize).toHaveBeenCalledTimes(1);
                } finally {
                  materialize.mockRestore();
                }
                compilation.deleteAsset('lazy.js');
                const variants = [
                  [
                    'mapped.js',
                    () =>
                      new OriginalSource(
                        "globalThis.value = '\u{1f680}';\n",
                        'original.ts',
                      ),
                  ],
                  ['plain.txt', () => new RawSource('plain\u00e9')],
                  [
                    'binary.dat',
                    () => new RawSource(Buffer.from([0, 255, 13])),
                  ],
                ];
                for (const [suffix, makeSource] of variants) {
                  for (const first of ['source', 'map', 'buffer', 'deferred']) {
                    const name = `${first}-${suffix}`;
                    compilation.emitAsset(name, makeSource(), {
                      development: true,
                      customField: 'preserved',
                    });
                    const raw = compilation.getAssetSource(name);
                    const original = Buffer.isBuffer(raw.source)
                      ? Buffer.from(raw.source)
                      : raw.source;
                    const expected = raw.map
                      ? new SourceMapSource(
                          raw.source,
                          'inmemory://from rust',
                          raw.map,
                        )
                      : new RawSource(raw.source);
                    const actual =
                      first === 'source'
                        ? compilation.assets[name]
                        : first === 'map'
                          ? compilation.getAsset(name).source
                          : compilation.getAssets().find((a) => a.name === name)
                              .source;
                    if (first === 'source') {
                      expect(actual.source()).toEqual(expected.source());
                    } else if (first === 'map') {
                      expect(actual.map({ columns: false })).toEqual(
                        expected.map({ columns: false }),
                      );
                    } else if (first === 'buffer') {
                      const buffer = actual.buffer();
                      expect(actual.buffer()).toBe(buffer);
                      buffer[0] = 42;
                      expected.buffer()[0] = 42;
                    }
                    if (
                      first === 'source' &&
                      Buffer.isBuffer(actual.source())
                    ) {
                      actual.source()[0] = 43;
                      expected.source()[0] = 43;
                    }
                    expect(compilation.getAssetSource(name).source).toEqual(
                      original,
                    );
                    compilation.setAssetSource(name, { source: 'replacement' });
                    expect(compilation.getAsset(name).info.customField).toBe(
                      'preserved',
                    );
                    expect(compilation.getAssetSource(name)).toEqual({
                      source: 'replacement',
                      map: undefined,
                    });
                    compilation.deleteAsset(name);
                    retained.push({ actual, expected });
                  }
                }
                compilation.setAssetSource('new.txt', { source: 'new' });
                expect(compilation.assets['new.txt'].source()).toBe('new');
                expect(() =>
                  compilation.setAssetSource('bad.js', {
                    source: 'text',
                    map: '{',
                  }),
                ).toThrow();
                expect(compilation.getAssetSource('bad.js')).toBeUndefined();
                const input = Buffer.from([1, 2, 3]);
                compilation.setAssetSource('copy.dat', {
                  source: input,
                  map: '{',
                });
                input[0] = 9;
                const copy = compilation.getAssetSource('copy.dat');
                expect(copy.map).toBeUndefined();
                expect(copy.source[0]).toBe(1);
                copy.source[0] = 8;
                expect(compilation.getAssetSource('copy.dat').source[0]).toBe(
                  1,
                );
              });
            });
          },
        },
      ],
    };
  },
  async check({ compiler }) {
    await new Promise((resolve, reject) => {
      compiler.close((error) => (error ? reject(error) : resolve()));
    });
    expect(this.retained.length).toBe(12);
    for (const { actual, expected } of this.retained) {
      expect(actual.source()).toEqual(expected.source());
      expect(actual.size()).toBe(expected.size());
      expect(actual.buffer()).toEqual(expected.buffer());
      for (const options of [
        undefined,
        { columns: false },
        { columns: true },
      ]) {
        expect(actual.map(options)).toEqual(expected.map(options));
        expect(actual.sourceAndMap(options)).toEqual(
          expected.sourceAndMap(options),
        );
        expect(stream(actual, options)).toEqual(stream(expected, options));
      }
      expect(digest(actual)).toBe(digest(expected));
      actual.buffer()[0] = 44;
      expected.buffer()[0] = 44;
      expect(actual.source()).toEqual(expected.source());
      expect(actual.sourceAndMap()).toEqual(expected.sourceAndMap());
      expect(digest(actual)).toBe(digest(expected));
    }
  },
};
