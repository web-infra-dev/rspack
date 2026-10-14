import { createHash } from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';
import { defineConfig, definePlugin } from '@rspack/cli';
import type { Readable } from 'node:stream';
import type { Source } from 'webpack-sources';

type CompressedSource = Source & {
  compressedBuffer: Buffer;
  contentHash: string;
  createReadStream(): Readable;
};

export default defineConfig({
  mode: 'development',
  devtool: 'source-map',
  node: { __dirname: false, __filename: false },
  externals: ['node:fs', 'node:path'],
  externalsType: 'commonjs',
  output: {
    filename: 'bundle0.js',
    sourceMapFilename: '[file].[contenthash:16].map',
    hashFunction: 'sha256',
    hashSalt: 'asset-source-test',
    compareBeforeEmit: false,
  },
  plugins: [
    definePlugin({
      apply(compiler) {
        const written = new Set<string>();
        compiler.hooks.initialize.tap('CompressedMap', () => {
          compiler.outputFileSystem = Object.assign(Object.create(fs), {
            writeFile(
              filename: string,
              bytes: Buffer,
              callback: fs.NoParamCallback,
            ) {
              // Map emission must reach createWriteStream without flattening the map.
              expect(filename.endsWith('.map')).toBe(false);
              fs.writeFile(filename, bytes, callback);
            },
            createWriteStream(filename: string) {
              written.add(filename);
              return fs.createWriteStream(filename);
            },
          });
        });
        compiler.hooks.thisCompilation.tap('CompressedMap', (compilation) => {
          compilation.hooks.processAssets.tapPromise(
            {
              name: 'CompressedMap',
              stage: compiler.webpack.Compilation.PROCESS_ASSETS_STAGE_REPORT,
            },
            async () => {
              const asset = compilation
                .getAssets()
                .find((asset) => asset.name.endsWith('.map'))!;
              const source = asset.source as CompressedSource;
              const encoded = source.compressedBuffer;
              expect(encoded.subarray(0, 4)).toEqual(
                Buffer.from([0x28, 0xb5, 0x2f, 0xfd]),
              );
              const bytes = source.buffer();
              expect(source.size()).toBe(bytes.length);
              expect(source.source()).toBe(bytes.toString());
              expect(source.contentHash).toMatch(/^[0-9a-f]{16}$/);
              const filenameHash = createHash('sha256')
                .update('asset-source-test')
                .update(bytes)
                .digest('hex')
                .slice(0, 16);
              expect(asset.name).toBe(`bundle0.js.${filenameHash}.map`);

              const streams = [
                source.createReadStream(),
                source.createReadStream(),
              ];
              const outputs = await Promise.all(
                streams.map(async (stream) => {
                  const chunks = [];
                  for await (const chunk of stream) chunks.push(chunk);
                  return Buffer.concat(chunks);
                }),
              );
              for (const output of outputs) expect(output).toEqual(bytes);
              // Mutating a JS snapshot must not modify the native frame.
              encoded.fill(0);
              expect(source.buffer()).toEqual(bytes);
            },
          );
        });
        compiler.hooks.afterEmit.tap('CompressedMap', (compilation) => {
          const map = compilation
            .getAssets()
            .find((asset) => asset.name.endsWith('.map'))!;
          expect([...written]).toContain(
            path.join(compiler.outputPath, map.name),
          );
        });
      },
    }),
  ],
});
