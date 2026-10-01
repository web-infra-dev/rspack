import { defineConfig } from '@rspack/cli';

export default defineConfig([
  {
    name: 'web',
    target: ['node', 'web'],
    output: {
      module: true,
    },
  },
  {
    name: 'node',
    target: ['node', 'web'],
    output: {
      module: true,
    },
  },
  {
    name: 'web',
    devtool: 'eval',
    target: ['node', 'web'],
    output: {
      module: true,
    },
  },
  {
    name: 'node',
    devtool: 'eval',
    target: ['node', 'web'],
    output: {
      module: true,
    },
  },
]);
