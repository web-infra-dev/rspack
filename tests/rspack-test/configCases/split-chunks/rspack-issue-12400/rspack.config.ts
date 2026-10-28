import { defineConfig, definePlugin } from '@rspack/cli';
import path from 'node:path';
import fs from 'node:fs';
import os from 'node:os';

const config = defineConfig({
  mode: 'development',
  target: 'web',
  devtool: false,
  entry: {
    app: './src/app',
    app2: './src/app2',
  },
  node: false,
  plugins: [
    definePlugin({
      apply(compiler) {
        compiler.hooks.done.tapAsync(
          'TestPlugin',
          ({ compilation }, callback) => {
            const chunks: string[] = [];

            Array.from(compilation.entrypoints.values()).forEach(
              (entrypoint) => {
                entrypoint.chunks.forEach((chunk) => {
                  // Simulate some processing on each chunk
                  chunks.push(
                    `${chunk.name}, ${Array.from(chunk.files).join(', ')}, ${Array.from(chunk.auxiliaryFiles).join(', ')}`,
                  );
                });
              },
            );

            fs.writeFileSync(
              path.join(compiler.outputPath, 'chunks-summary.txt'),
              chunks.join(os.EOL),
              'utf-8',
            );
            callback();
          },
        );
      },
    }),
  ],
  optimization: {
    runtimeChunk: true,
    chunkIds: 'named',
    moduleIds: 'named',
    splitChunks: {
      name(_module, chunks) {
        return chunks.map((item) => item.name).join('~');
      },
      minSize: 0,
      chunks: 'all',
    },
  },
  externals: {
    fs: 'node-commonjs fs',
    path: 'node-commonjs path',
    underscore: {
      root: 'fs',
    },
    jquery: {
      root: 'fs',
    },
  },
  output: {
    filename: '[name].js',
  },
});

export default config;
