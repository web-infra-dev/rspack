const assert = require('assert');

function assertRealChunk(data, name) {
  assert(data.chunk);
  assert.strictEqual(data.chunk.name, name);
  assert.strictEqual(typeof data.chunk.getEntryOptions, 'function');
  assert.strictEqual(typeof data.chunk.canBeInitial, 'function');
}

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  mode: 'development',
  entry: {
    a: './a',
    b: {
      import: './b',
      filename: (data) => {
        assertRealChunk(data, 'b');
        return data.chunk.name + data.chunk.name + data.chunk.name + '.js';
      },
    },
  },
  output: {
    filename: (data) => {
      assertRealChunk(data, data.chunk.name);
      return data.chunk.name + data.chunk.name + '.js';
    },
    chunkFilename: (data) => {
      assertRealChunk(data, data.chunk.name);
      return data.chunk.name + data.chunk.name + '.js';
    },
  },
};
