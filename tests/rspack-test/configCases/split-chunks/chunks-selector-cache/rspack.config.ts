import { defineConfig } from '@rspack/cli';
import { type Chunk, type Configuration } from '@rspack/core';

const outputs = new Map<boolean, Map<string, Buffer>>();

function createConfig(native: boolean): Configuration {
  let calls = 0;
  const seen = new Set<Chunk>();
  const selected = /^[abc]$/;
  return {
    mode: 'development',
    target: 'node',
    entry: { a: './index.js', b: './index.js', c: './index.js' },
    devtool: false,
    output: { filename: '[name].js', chunkFilename: '[name].js' },
    optimization: {
      minimize: false,
      concatenateModules: false,
      usedExports: false,
      splitChunks: {
        minSize: 0,
        cacheGroups: {
          default: false,
          defaultVendors: false,
          shared: {
            test: /shared-\d+\.js$/,
            name: 'shared',
            minChunks: 2,
            enforce: true,
            chunks: native
              ? selected
              : (chunk: Chunk) => {
                  calls++;
                  seen.add(chunk);
                  return selected.test(chunk.name ?? '');
                },
          },
        },
      },
    },
    plugins: [
      {
        apply(compiler) {
          compiler.hooks.afterEmit.tap('CheckSelectorCache', (compilation) => {
            // One eligible set {a,b,c}: at most one call per member.
            if (!native) expect(calls).toBeLessThanOrEqual(seen.size);
            outputs.set(
              native,
              new Map(
                compilation
                  .getAssets()
                  .filter((asset) => asset.name.endsWith('.js'))
                  .map((asset) => [
                    asset.name,
                    Buffer.from(asset.source.source()),
                  ]),
              ),
            );
            if (outputs.size === 2)
              expect(outputs.get(false)).toEqual(outputs.get(true));
          });
        },
      },
    ],
  };
}

export default defineConfig([createConfig(false), createConfig(true)]);
