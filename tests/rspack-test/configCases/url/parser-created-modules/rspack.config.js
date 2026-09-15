const path = require('node:path');

module.exports = {
  mode: 'development',
  devtool: false,
  target: 'web',
  output: {
    filename: 'bundle.js',
    chunkFilename: 'url-[id].js',
    assetModuleFilename: '[name][ext]',
    publicPath: '/assets/',
  },
  module: {
    rules: [
      { test: /target\.js$/, dependency: 'url', type: 'javascript/auto' },
    ],
  },
  plugins: [
    (compiler) => {
      compiler.hooks.compilation.tap(
        'CheckParserModules',
        (compilation, { normalModuleFactory }) => {
          const builds = [];
          const factorizations = [];
          compilation.hooks.buildModule.tap('CheckParserModules', (module) => {
            builds.push(module.resource && path.basename(module.resource));
          });
          normalModuleFactory.hooks.beforeResolve.tap(
            'CheckParserModules',
            ({ request }) => {
              factorizations.push(request);
            },
          );
          compilation.hooks.finishModules.tap(
            'CheckParserModules',
            (modules) => {
              for (const target of ['target.js', 'target.txt', 'nested.txt']) {
                expect(builds.filter((name) => name === target)).toHaveLength(
                  1,
                );
              }
              expect(
                factorizations.filter((request) => request === './target.js'),
              ).toHaveLength(4);
              expect(
                factorizations.filter((request) => request === './target.txt'),
              ).toHaveLength(2);
              expect(
                factorizations.filter((request) => request === './nested.txt'),
              ).toHaveLength(1);
              const targets = [];
              for (const issuer of modules) {
                if (
                  !issuer.resource ||
                  !['a.js', 'b.js'].includes(path.basename(issuer.resource))
                )
                  continue;
                expect(issuer.blocks).toHaveLength(2);
                for (const block of issuer.blocks)
                  targets.push(
                    compilation.moduleGraph.getModule(block.dependencies[0]),
                  );
              }
              expect(targets).toHaveLength(4);
              expect(
                new Set(targets.map((module) => module.identifier())).size,
              ).toBe(1);
            },
          );
          compilation.hooks.processAssets.tap('CheckParserModules', () => {
            expect(compilation.getAsset('nested.txt')).toBeDefined();
          });
        },
      );
    },
  ],
};
