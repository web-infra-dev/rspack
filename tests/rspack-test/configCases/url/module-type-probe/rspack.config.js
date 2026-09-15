const { NormalModuleReplacementPlugin } = require('@rspack/core');
const path = require('node:path');

module.exports = {
  mode: 'development',
  devtool: false,
  target: 'web',
  output: {
    filename: 'bundle0.js',
    chunkFilename: 'url-[id].js',
    publicPath: '/assets/',
    assetModuleFilename: '[name][ext]',
  },
  module: {
    rules: [
      { test: /resource\.txt$/, dependency: 'url', type: 'asset/resource' },
      { test: /inline\.txt$/, dependency: 'url', type: 'asset/inline' },
      { test: /[/\\]source\.txt$/, dependency: 'url', type: 'asset/source' },
      {
        test: /custom\.txt$/,
        dependency: 'url',
        type: 'asset/resource',
        generator: { publicPath: '/custom/', filename: '[name][ext]' },
      },
      { test: /target\.js$/, dependency: 'url', type: 'javascript/auto' },
      { test: /target\.txt$/, dependency: 'url', type: 'asset/resource' },
    ],
  },
  plugins: [
    new NormalModuleReplacementPlugin(/replace-target\.txt$/, './target.js'),
    new NormalModuleReplacementPlugin(/replace-target\.js$/, './target.txt'),
    (compiler) => {
      compiler.hooks.compilation.tap(
        'CheckTypeProbe',
        (compilation, { normalModuleFactory }) => {
          const factorizations = [];
          normalModuleFactory.hooks.beforeResolve.tap(
            { name: 'CheckTypeProbe', stage: -1000 },
            ({ request }) => {
              factorizations.push(request);
            },
          );
          const builds = [];
          compilation.hooks.buildModule.tap('CheckTypeProbe', (module) => {
            builds.push(module.resource);
          });
          compilation.hooks.finishModules.tap('CheckTypeProbe', (modules) => {
            const issuer = [...modules].find(
              (module) => module.rawRequest === './index.js',
            );
            expect(issuer.blocks).toHaveLength(1);
            for (const request of [
              './replace-target.txt',
              './replace-target.js',
            ]) {
              expect(
                factorizations.filter((value) => value === request),
              ).toHaveLength(1);
            }
            expect(
              issuer.dependencies.filter((dep) => dep.type === 'new URL()'),
            ).toHaveLength(1);
            const blockTarget = compilation.moduleGraph.getModule(
              issuer.blocks[0].dependencies[0],
            );
            expect(blockTarget.type).toBe('javascript/auto');
            const assetIssuer = [...modules].find(
              (module) => module.rawRequest === './assets.js',
            );
            expect(assetIssuer.blocks).toHaveLength(1);
            expect(
              assetIssuer.blocks[0].dependencies.filter(
                (dep) => dep.type === 'new URL()',
              ),
            ).toHaveLength(4);
            expect(
              assetIssuer.dependencies.filter(
                (dep) => dep.type === 'new URL()',
              ),
            ).toHaveLength(4);
            for (const target of ['target.js', 'target.txt']) {
              expect(
                builds.filter(
                  (resource) => resource && path.basename(resource) === target,
                ),
              ).toHaveLength(1);
            }
          });
          compilation.hooks.processAssets.tap('CheckUrlAssets', () => {
            expect(compilation.getAsset('resource.txt')).toBeDefined();
            expect(compilation.getAsset('custom.txt')).toBeDefined();
            expect(compilation.getAsset('inline.txt')).toBeUndefined();
            expect(compilation.getAsset('source.txt')).toBeUndefined();
          });
        },
      );
    },
  ],
};
