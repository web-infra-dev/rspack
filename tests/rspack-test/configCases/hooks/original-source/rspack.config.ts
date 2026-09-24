import { createHash } from 'node:crypto';
import { defineConfig } from '@rspack/cli';
import { type Compiler, sources } from '@rspack/core';
import type { Source } from 'webpack-sources';

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
                  const map = module._originalSource()!.map;
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
                  for (const options of [
                    undefined,
                    { maps: false },
                    { source: false },
                  ]) {
                    source.clearCache(options);
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
