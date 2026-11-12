import { defineConfig } from '@rspack/cli';

const base = defineConfig({
  optimization: {
    concatenateModules: true,
  },
  target: 'es2020',
});

export default defineConfig([
  {
    ...base,
    name: 'module-avoidEntryIife-false',
    output: {
      filename: 'module-avoidEntryIife-false.mjs',
      module: true,
    },
    optimization: {
      ...base.optimization,
      avoidEntryIife: false,
    },
  },
  {
    ...base,
    name: 'module-avoidEntryIife-true',
    output: {
      module: true,
      filename: 'module-avoidEntryIife-true.mjs',
    },
    optimization: {
      ...base.optimization,
      avoidEntryIife: true,
    },
  },
  {
    name: 'test-output',
    entry: './test.js',
    output: {
      filename: 'test.js',
    },
  },
]);
