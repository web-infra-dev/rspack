import { strict as assert } from 'node:assert';

class MyEntryOptionPlugin {
  apply(compiler) {
    compiler.hooks.entryOption.tap('MyEntryOptionPlugin', (context, entry) => {
      assert(context === config.context, 'Context is not equal.');
      assert.deepStrictEqual(
        entry,
        Object.keys(config.entry).reduce((acc, key) => {
          acc[key] = { import: [config.entry[key]] };
          return acc;
        }, {}),
        'Entry is not strictly equal.',
      );
    });
  }
}

/** @type {import("@rspack/core").Configuration} */
const config = {
  context: import.meta.dirname,
  mode: 'development',
  entry: {
    main: './src/index.js',
    test: './src/index2.js',
  },
  output: {
    filename: '[name].js',
  },
  plugins: [new MyEntryOptionPlugin()],
};
export default config;
