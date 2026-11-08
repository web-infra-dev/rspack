import path from 'node:path';
import rspackconfig from '../0-container-full/rspack.config.ts';
import { defineConfig } from '@rspack/cli';
import { container } from '@rspack/core';

const distRoot = path.resolve(
  import.meta.dirname,
  '../../../js/serial/container-1-5/4-async-startup-runtime-chunk-single',
);
const remoteOut = path.join(distRoot, '0-container-full');
const remoteContext = path.resolve(import.meta.dirname, '../0-container-full');

// Reuse the real remote container config so the case exercises emitted remotes.
const remoteConfigs = rspackconfig.map((config) => {
  const isModule = config.output?.module;
  return defineConfig({
    ...config,
    context: remoteContext,
    output: {
      ...config.output,
      path: remoteOut,
      filename: isModule ? 'module/[name].mjs' : '[name].js',
      chunkFilename: isModule ? 'module/[id].mjs' : '[id].js',
    },
  });
});
// eslint-disable-next-line node/no-unpublished-require
const { ModuleFederationPlugin } = container;

const common = defineConfig({
  entry: {
    main: './index.js',
  },
  optimization: {
    runtimeChunk: 'single',
  },
});

const commonMF = {
  runtime: false as const,
  exposes: {
    './ComponentB': './ComponentB',
    './ComponentC': './ComponentC',
  },
  shared: ['@rspack/mocked-react'],
};

export default defineConfig([
  ...remoteConfigs,
  // Host bundles under test (CJS + ESM)
  {
    ...common,
    target: 'async-node',
    output: {
      filename: '[name].js',
      uniqueName: '4-async-startup-runtime-chunk-single',
      chunkLoading: 'async-node',
    },
    plugins: [
      new ModuleFederationPlugin({
        name: 'container',
        library: { type: 'commonjs-module' },
        filename: 'container.js',
        remotes: {
          containerA: './0-container-full/container.js',
          containerB: './container.js',
        },
        ...commonMF,
        experiments: {
          asyncStartup: true,
        },
      }),
    ],
  },
  {
    ...common,
    output: {
      filename: 'module/[name].mjs',
      uniqueName: '4-async-startup-runtime-chunk-single-mjs',
    },
    plugins: [
      new ModuleFederationPlugin({
        name: 'container',
        library: { type: 'module' },
        filename: 'module/container.mjs',
        remotes: {
          // NOTE: this resolves from the host ESM output directory (module/)
          // so we need a single ../ to reach the collocated remote outputs.
          containerA: '../0-container-full/module/container.mjs',
          containerB: './container.mjs',
        },
        ...commonMF,
        experiments: {
          asyncStartup: true,
        },
      }),
    ],
    target: 'node14',
  },
]);
