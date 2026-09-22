import { defineConfig } from '@rspack/cli';
import { type Compiler } from '@rspack/core';

class MyEntryOptionPlugin {
  apply(compiler: Compiler) {
    compiler.hooks.entryOption.tap('MyEntryOptionPlugin', (context, entry) => {
      expect(context, 'Context is not equal.').toBe(config.context);
      expect(entry, 'Entry is not strictly equal.').toStrictEqual(
        Object.fromEntries(
          Object.entries(config.entry).map(([key, value]) => [
            key,
            { import: [value] },
          ]),
        ),
      );
    });
  }
}

const config = {
  context: import.meta.dirname,
  mode: 'development' as const,
  entry: {
    main: './src/index.js',
    test: './src/index2.js',
  },
  output: {
    filename: '[name].js',
  },
  plugins: [new MyEntryOptionPlugin()],
};

export default defineConfig(config);
