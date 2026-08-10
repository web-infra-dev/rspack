const fs = require('fs');
const path = require('path');
const { rspack } = require('@rspack/core');

const entries = [
  './index-asset.js',
  './index-css.js',
  './index-wasm.js',
  './index-js.js',
  './index-inline.js',
  './index-shared.js',
  './index-split.js',
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
        test: /target\.png$/,
        type: 'asset/resource',
      },
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
        test: /target(?:-split)?\.js$/,
        dependency: 'url',
        type: 'javascript/auto',
      },
      {
        resourceQuery: /inline/,
        dependency: 'url',
        type: 'asset/inline',
      },
    ],
  },
  optimization:
    i === 6
      ? {
          splitChunks: {
            chunks: 'all',
            minSize: 0,
            cacheGroups: {
              shared: {
                test: /shared-split/,
                name: 'shared',
                enforce: true,
              },
            },
          },
        }
      : undefined,
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
              const expectedBlockCount = i === 5 ? 2 : 1;
              expect(originModule.blocks).toHaveLength(expectedBlockCount);
              for (const block of originModule.blocks) {
                expect(block.dependencies).toHaveLength(1);
                expect(block.dependencies[0].type).toBe('new URL()');
              }

              const mainSource = compilation
                .getAsset(`${i}/main.js`)
                .source.source()
                .toString();
              const expectedOutput = [
                /\/assets\/0\/target\.png/,
                /1\/url-[^" ]+\.css/,
                /2\/[^" ]+\.wasm/,
                /3\/url-[^" ]+\.js/,
                /data:image\/png;base64,/,
                undefined,
                /6\/url-[^" ]+\.js/,
              ][i];
              if (expectedOutput) {
                expect(mainSource).toMatch(expectedOutput);
              } else {
                const targetUrls = mainSource.match(/5\/url-[^" ]+\.js/g);
                expect(targetUrls).toHaveLength(2);
                expect(new Set(targetUrls).size).toBe(2);
              }
              if (i === 6) {
                const asyncEntryAsset = compilation
                  .getAssets()
                  .find((asset) =>
                    /6\/url-(?!shared\.js$)[^/]+\.js$/.test(asset.name),
                  );
                expect(asyncEntryAsset).toBeDefined();
                const asyncEntrySource = asyncEntryAsset.source
                  .source()
                  .toString();
                expect(asyncEntrySource).toMatch(/shared/);
              }

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
