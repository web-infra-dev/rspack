import { defineConfig } from '@rspack/cli';
import { type Chunk, type Compiler, type PathData } from '@rspack/core';

function isEntryChunk(chunk: Chunk) {
  for (const group of chunk.groupsIterable) {
    if (group.isInitial() && group.getEntrypointChunk() === chunk) {
      return true;
    }
  }
  return false;
}

function filename(pathData: PathData) {
  const chunk = pathData.chunk as Chunk;
  expect(pathData.chunk).toBeTruthy();
  expect(typeof chunk.groupsIterable[Symbol.iterator]).toBe('function');
  expect(chunk.canBeInitial()).toBe(true);

  return isEntryChunk(chunk) ? '[name].js' : '[name]-initial-chunk.js';
}

class Plugin {
  apply(compiler: Compiler) {
    compiler.hooks.done.tap('filename-function-entry-chunk', (stats) => {
      const assetNames = stats
        .toJson({ all: false, assets: true })
        .assets?.map((asset) => asset.name)
        .sort();

      expect(assetNames).toStrictEqual([
        'a.js',
        'b.js',
        'shared-initial-chunk.js',
      ]);
    });
  }
}

export default defineConfig({
  mode: 'development',
  entry: {
    a: './a',
    b: './b',
  },
  output: {
    filename,
  },
  optimization: {
    chunkIds: 'named',
    splitChunks: {
      chunks: 'initial',
      minSize: 0,
      cacheGroups: {
        shared: {
          name: 'shared',
          minChunks: 2,
          enforce: true,
        },
      },
    },
  },
  plugins: [new Plugin()],
});
