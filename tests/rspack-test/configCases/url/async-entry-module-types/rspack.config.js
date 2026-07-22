const fs = require('fs');
const path = require('path');
const { rspack } = require('@rspack/core');

const entries = [
  './index-asset.js',
  './index-css.js',
  './index-wasm.js',
  './index-js.js',
];

/** @type {import("@rspack/core").Configuration[]} */
module.exports = entries.map((entry, i) => ({
  mode: 'development',
  devtool: false,
  target: 'web',
  entry: {
    main: entry,
  },
  experiments: {
    asyncWebAssembly: true,
  },
  output: {
    filename: `${i}/[name].js`,
    chunkFilename: `${i}/url-[id].js`,
    cssChunkFilename: `${i}/url-[id].css`,
    assetModuleFilename: `${i}/[name][ext]`,
    webassemblyModuleFilename: `${i}/[id].wasm`,
    publicPath: '/assets/',
  },
  module: {
    rules: [
      {
        test: /target\.css$/,
        dependency: 'url',
        type: 'css',
      },
      {
        test: /test\.wasm$/,
        dependency: 'url',
        type: 'webassembly/async',
      },
      {
        test: /target\.js$/,
        dependency: 'url',
        type: 'javascript/auto',
      },
    ],
  },
  plugins: [
    {
      apply(compiler) {
        compiler.hooks.compilation.tap('Test', (compilation) => {
          compilation.hooks.processAssets.tap(
            {
              name: 'copy-test-runner',
              stage: rspack.Compilation.PROCESS_ASSETS_STAGE_ADDITIONAL,
            },
            () => {
              const originModule = Array.from(compilation.modules).find(
                (module) => module.rawRequest === entry,
              );
              expect(originModule.blocks).toHaveLength(1);
              expect(originModule.blocks[0].dependencies).toHaveLength(1);
              expect(originModule.blocks[0].dependencies[0].type).toBe(
                'new URL()',
              );

              compilation.emitAsset(
                'test.js',
                new rspack.sources.RawSource(
                  fs.readFileSync(path.resolve(__dirname, 'test.js')),
                ),
              );
            },
          );
        });
      },
    },
  ],
}));
