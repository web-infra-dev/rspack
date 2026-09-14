const { NormalModuleReplacementPlugin } = require('@rspack/core');
const path = require('node:path');

module.exports = {
  mode: 'development',
  devtool: false,
  target: 'web',
  output: {
    filename: 'bundle.js',
    chunkFilename: 'url-[id].js',
    publicPath: '/assets/',
    assetModuleFilename: '[name][ext]',
  },
  module: {
    rules: [
      { test: /target\.js$/, dependency: 'url', type: 'javascript/auto' },
      { test: /target\.txt$/, dependency: 'url', type: 'asset/resource' },
    ],
  },
  plugins: [
    new NormalModuleReplacementPlugin(/replace-target\.txt$/, './target.js'),
    new NormalModuleReplacementPlugin(/replace-target\.js$/, './target.txt'),
    (compiler) => {
      compiler.hooks.compilation.tap('CheckTypeProbe', (compilation) => {
        const builds = [];
        compilation.hooks.buildModule.tap('CheckTypeProbe', (module) => {
          builds.push(module.resource);
        });
        compilation.hooks.finishModules.tap('CheckTypeProbe', (modules) => {
          const issuer = [...modules].find(
            (module) => module.rawRequest === './index.js',
          );
          expect(issuer.blocks).toHaveLength(1);
          expect(
            issuer.dependencies.filter((dep) => dep.type === 'new URL()'),
          ).toHaveLength(1);
          const blockTarget = compilation.moduleGraph.getModule(
            issuer.blocks[0].dependencies[0],
          );
          expect(blockTarget.type).toBe('javascript/auto');
          for (const target of ['target.js', 'target.txt']) {
            expect(
              builds.filter(
                (resource) => resource && path.basename(resource) === target,
              ),
            ).toHaveLength(1);
          }
        });
      });
    },
  ],
};
