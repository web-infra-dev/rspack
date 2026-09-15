let sawEntryChunk = false;
let sawAsyncChunk = false;
let sawNumberChunkId = false;

function pickName(pathData) {
  const chunk = pathData.chunk;

  if (!chunk) {
    return '[name].js';
  }

  const groupsIterable = chunk.groupsIterable ?? chunk._groupsIterable;
  if (
    chunk.constructor?.name !== 'Chunk' ||
    typeof chunk.getEntryOptions !== 'function' ||
    typeof groupsIterable?.[Symbol.iterator] !== 'function'
  ) {
    throw new Error('pathData.chunk is not a real Chunk instance');
  }
  if (typeof chunk.id === 'number') {
    sawNumberChunkId = true;
  }
  const isEntryChunk = [...groupsIterable].some(
    (group) => group.isInitial() && group.getEntrypointChunk() === chunk,
  );
  if (isEntryChunk) {
    sawEntryChunk = true;
  } else {
    sawAsyncChunk = true;
  }
  return isEntryChunk ? 'entry-[name].js' : 'async-[name].js';
}

class AssertChunkPlugin {
  apply(compiler) {
    compiler.hooks.compilation.tap('AssertChunkPlugin', () => {
      sawEntryChunk = false;
      sawAsyncChunk = false;
      sawNumberChunkId = false;
    });
    compiler.hooks.done.tap('AssertChunkPlugin', () => {
      expect(sawEntryChunk).toBe(true);
      expect(sawAsyncChunk).toBe(true);
      expect(sawNumberChunkId).toBe(true);
    });
  }
}

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  entry: './index.js',
  output: {
    filename: pickName,
    chunkFilename: pickName,
  },
  optimization: {
    chunkIds: 'natural',
  },
  plugins: [new AssertChunkPlugin()],
};
