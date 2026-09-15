const { Compilation, sources } = require('@rspack/core');

class CircularHashDependencyPlugin {
  apply(compiler) {
    compiler.hooks.thisCompilation.tap(
      'CircularHashDependencyPlugin',
      (compilation) => {
        compilation.hooks.processAssets.tap(
          {
            name: 'CircularHashDependencyPlugin',
            stage: Compilation.PROCESS_ASSETS_STAGE_OPTIMIZE_HASH - 1,
          },
          () => {
            const source = 'a:aaaaaaaa;b:bbbbbbbb';
            compilation.emitAsset(
              'a.aaaaaaaa.js',
              new sources.RawSource(source),
              { contenthash: ['aaaaaaaa', 'cccccccc'] },
            );
            compilation.emitAsset(
              'b.bbbbbbbb.js',
              new sources.RawSource(source),
              { contenthash: 'bbbbbbbb' },
            );
          },
        );
      },
    );
  }
}

/** @type {import('@rspack/core').Configuration} */
module.exports = {
  mode: 'production',
  entry: './index.js',
  optimization: {
    minimize: false,
    realContentHash: true,
  },
  plugins: [new CircularHashDependencyPlugin()],
};
