const { rspack } = require('@rspack/core');

class CheckUrlEntryBlocksPlugin {
  apply(compiler) {
    compiler.hooks.compilation.tap(
      'CheckUrlEntryBlocksPlugin',
      (compilation) => {
        compilation.hooks.processAssets.tap(
          {
            name: 'CheckUrlEntryBlocksPlugin',
            stage: rspack.Compilation.PROCESS_ASSETS_STAGE_ADDITIONS,
          },
          () => {
            const originModule = Array.from(compilation.modules).find(
              (module) => module.rawRequest === './index.js',
            );
            expect(originModule).toBeDefined();
            expect(originModule.blocks).toHaveLength(2);
            for (const block of originModule.blocks) {
              expect(block.dependencies).toHaveLength(1);
              expect(block.dependencies[0].type).toBe('new URL()');
            }
            expect(
              originModule.dependencies.filter(
                (dependency) => dependency.type === 'new URL()',
              ),
            ).toHaveLength(0);

            const assets = compilation.getAssets().map((asset) => asset.name);
            expect(
              assets.filter((asset) => asset.endsWith('.js')),
            ).toHaveLength(2);
            expect(
              assets.filter((asset) => asset.endsWith('.css')),
            ).toHaveLength(1);
          },
        );
      },
    );
  }
}

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  mode: 'development',
  devtool: false,
  target: 'web',
  output: {
    filename: 'bundle.js',
    chunkFilename: 'url-[id].js',
    cssChunkFilename: 'url-[id].css',
    publicPath: '/assets/',
  },
  module: {
    rules: [
      {
        test: /target\.js$/,
        dependency: 'url',
        type: 'javascript/auto',
      },
      {
        test: /target\.css$/,
        dependency: 'url',
        type: 'css',
      },
    ],
  },
  plugins: [new CheckUrlEntryBlocksPlugin()],
};
