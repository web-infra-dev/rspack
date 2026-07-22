const assert = require('assert');

function isEntryChunk(chunk) {
  for (const group of chunk.groupsIterable) {
    if (group.isInitial() && group.getEntrypointChunk() === chunk) {
      return true;
    }
  }
  return false;
}

function filename(pathData) {
  assert(pathData.chunk);
  assert.strictEqual(
    typeof pathData.chunk.groupsIterable[Symbol.iterator],
    'function',
  );
  assert.strictEqual(pathData.chunk.canBeInitial(), true);

  return isEntryChunk(pathData.chunk) ? '[name].js' : '[name]-initial-chunk.js';
}

class Plugin {
  apply(compiler) {
    compiler.hooks.done.tap('filename-function-entry-chunk', (stats) => {
      const assetNames = stats
        .toJson({ all: false, assets: true })
        .assets.map((asset) => asset.name)
        .sort();

      assert.deepStrictEqual(assetNames, [
        'a.js',
        'b.js',
        'shared-initial-chunk.js',
      ]);
    });
  }
}

/** @type {import("@rspack/core").Configuration} */
module.exports = {
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
};
