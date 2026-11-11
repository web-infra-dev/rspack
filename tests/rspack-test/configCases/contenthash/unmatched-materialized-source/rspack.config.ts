import { readFileSync } from 'node:fs';
import path from 'node:path';
import { defineConfig } from '@rspack/cli';
import { rspack, type Compiler } from '@rspack/core';

const oldHash = 'deadbeef';
const binaryName = `binary.${oldHash}.js`;
const banner = 'banner';

class TestPlugin {
  private readonly matched: boolean;

  constructor(matched: boolean) {
    this.matched = matched;
  }

  apply(compiler: Compiler) {
    const payload = Buffer.concat([
      Buffer.from([0xff, 0]),
      Buffer.from(this.matched ? oldHash : 'payload'),
    ]);
    compiler.hooks.thisCompilation.tap('Test', (compilation) => {
      compilation.hooks.processAssets.tap(
        {
          name: 'Test',
          stage:
            compiler.webpack.Compilation.PROCESS_ASSETS_STAGE_ADDITIONS - 1,
        },
        () => {
          compilation.updateAsset(
            binaryName,
            new compiler.webpack.sources.RawSource(payload),
            { contenthash: oldHash },
          );
        },
      );
    });
    compiler.hooks.afterEmit.tap('Test', (compilation) => {
      // BannerPlugin wraps the buffer in a native ConcatSource. Its text view
      // is lossy, while its byte writer preserves 0xff when no hash is replaced.
      const prefix = Buffer.from(`${banner}\n`);
      const hashInput = this.matched
        ? Buffer.from(`${banner}\n${payload.toString().replace(oldHash, '')}`)
        : Buffer.concat([prefix, payload]);
      const hasher = compiler.webpack.util.createHash('xxhash64');
      hasher.update(hashInput);
      const hash = hasher.digest('hex').slice(0, oldHash.length);
      const outputName = `binary.${hash}.js`;
      const expected = this.matched
        ? Buffer.from(`${banner}\n${payload.toString().replace(oldHash, hash)}`)
        : hashInput;
      const asset = compilation.getAsset(outputName);
      expect(asset).toBeDefined();
      expect(asset!.source.source()).toBe(expected.toString());
      expect(readFileSync(path.join(compiler.outputPath, outputName))).toEqual(
        expected,
      );
    });
  }
}

export default defineConfig(
  [false, true].map((matched) => ({
    mode: 'production',
    devtool: false,
    entry: { main: './index.js', binary: './index.js' },
    output: {
      filename: ({ chunk }) =>
        chunk?.name === 'binary' ? binaryName : 'bundle.js',
      hashFunction: 'xxhash64',
    },
    optimization: { minimize: false, realContentHash: true },
    plugins: [
      new TestPlugin(matched),
      new rspack.BannerPlugin({ banner, raw: true, test: /^binary\./ }),
    ],
  })),
);
