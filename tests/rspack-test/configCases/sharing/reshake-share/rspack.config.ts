import path from 'node:path';
import { defineConfig } from '@rspack/cli';
import { sharing, type Compiler, type SharedObject } from '@rspack/core';

const { ProvideSharedPlugin, TreeShakingSharedPlugin } = sharing;

// Mimic @module-federation/rspack, whose explicit plugin name differs from its
// constructor name. The plugin must not be inherited by shared child compilers.
const RspackModuleFederationPlugin = class ModuleFederationPlugin {
  declare name: string;

  constructor() {
    this.name = 'RspackModuleFederationPlugin';
  }

  apply(compiler: Compiler) {
    if (compiler.options.name === 'mf-shared-compiler') {
      throw new Error(
        'RspackModuleFederationPlugin should not be applied to shared child compilers',
      );
    }
  }
};

const shared: SharedObject = {
  'ui-lib': {
    version: '1.0.0',
    treeShaking: {
      mode: 'runtime-infer',
      usedExports: ['Badge', 'MessagePro'],
    },
    requiredVersion: '^1.0.0',
  },
  'ui-lib-dep': {
    version: '1.0.0',
    treeShaking: {
      mode: 'runtime-infer',
      usedExports: ['Message'],
    },
    requiredVersion: '^1.0.0',
  },
};

export default defineConfig({
  externals: {
    fs: 'node-commonjs fs',
    path: 'node-commonjs path',
  },
  optimization: {
    minimize: true,
    chunkIds: 'named',
    moduleIds: 'named',
  },
  output: {
    chunkFilename: '[id].js',
  },
  plugins: [
    new RspackModuleFederationPlugin(),
    new ProvideSharedPlugin({
      provides: {
        'ui-lib': {
          shareKey: 'ui-lib',
          version: '1.0.0',
          requiredVersion: '^1.0.0',
          treeShakingMode: 'runtime-infer',
        },
        'ui-lib-dep': {
          shareKey: 'ui-lib-dep',
          version: '1.0.0',
          requiredVersion: '^1.0.0',
          treeShakingMode: 'runtime-infer',
        },
      },
      enhanced: true,
    }),
    new TreeShakingSharedPlugin({
      secondary: true,
      mfConfig: {
        name: 'secondary_tree_shaking_share',
        library: {
          type: 'commonjs2',
        },
        shared,
        treeShakingSharedExcludePlugins: ['ProvideSharedPlugin'],
        treeShakingSharedPlugins: [
          path.resolve(import.meta.dirname, './CustomPlugin.js'),
        ],
      },
    }),
  ],
});
