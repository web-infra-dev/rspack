import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

const {
  experiments: { RslibPlugin },
} = rspack;

const baseConfig = (index: number, mjs = false) =>
  defineConfig({
    entry: {
      index: {
        import: './index.js',
        filename: `bundle${index}${mjs ? '.mjs' : '.js'}`,
      },
    },
    target: 'node',
    node: {
      __filename: false,
      __dirname: false,
    },
  });

export default defineConfig([
  // CJS output
  {
    ...baseConfig(0),
    externals: {
      react: 'react',
    },
    output: {
      library: {
        type: 'commonjs',
      },
    },
    plugins: [new RslibPlugin()],
  },
  // ESM output (without EsmLibraryPlugin)
  {
    ...baseConfig(1, true),
    externals: {
      react: 'module react',
    },
    output: {
      module: true,
      library: {
        type: 'modern-module',
      },
    },
    plugins: [new RslibPlugin()],
  },
  // ESM output (with EsmLibraryPlugin)
  {
    ...baseConfig(2, true),
    externals: {
      react: 'module react',
    },
    output: {
      module: true,
      library: {
        type: 'modern-module',
      },
    },
    plugins: [new RslibPlugin()],
  },
  // Test entry
  {
    entry: './test.js',
    target: 'node',
    output: {
      filename: 'bundle3.js',
    },
    node: {
      __filename: false,
      __dirname: false,
    },
  },
]);
