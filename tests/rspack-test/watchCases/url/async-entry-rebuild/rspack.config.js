const { rspack } = require('@rspack/core');

class CheckUrlEntryBlocksPlugin {
  apply(compiler) {
    let buildIndex = 0;
    let issuerBuilds = 0;
    compiler.hooks.compilation.tap(
      'CheckUrlEntryBlocksPlugin',
      (compilation) => {
        const currentBuild = buildIndex++;
        const isAsset = currentBuild === 2 || currentBuild === 4;
        compilation.hooks.buildModule.tap(
          'CheckUrlEntryBlocksPlugin',
          (module) => {
            if (module.rawRequest === './index.js') issuerBuilds++;
          },
        );
        compilation.hooks.finishModules.tap(
          'CheckUrlEntryBlocksPlugin',
          (modules) => {
            expect(issuerBuilds).toBe(currentBuild + 1);
            const issuer = [...modules].find(
              (module) => module.rawRequest === './index.js',
            );
            expect(issuer).toBeDefined();
            expect(issuer.blocks).toHaveLength(isAsset ? 0 : 1);
            const directUrls = issuer.dependencies.filter(
              (dep) => dep.type === 'new URL()',
            );
            expect(directUrls).toHaveLength(isAsset ? 1 : 0);
            const dependencies = isAsset
              ? directUrls
              : issuer.blocks[0].dependencies;
            expect(dependencies).toHaveLength(1);
            expect(dependencies[0].type).toBe('new URL()');
            expect(
              compilation.moduleGraph.getModule(dependencies[0]).type,
            ).toBe(isAsset ? 'asset/resource' : 'javascript/auto');
          },
        );
        compilation.hooks.processAssets.tap(
          {
            name: 'CheckUrlEntryBlocksPlugin',
            stage: rspack.Compilation.PROCESS_ASSETS_STAGE_ADDITIONS,
          },
          () => {
            const scripts = compilation
              .getAssets()
              .filter((asset) => asset.name.endsWith('.js'));
            expect(scripts).toHaveLength(isAsset ? 1 : 2);
            if (isAsset) {
              expect(
                compilation
                  .getAsset('target.txt')
                  .source.source()
                  .toString()
                  .trim(),
              ).toBe(
                currentBuild === 4
                  ? 'updated asset content'
                  : 'URL target asset',
              );
            } else {
              const entry = scripts.find((asset) =>
                asset.name.startsWith('url-'),
              );
              expect(entry.source.source().toString()).toContain(
                ['initial', 'updated', undefined, 'restored'][currentBuild],
              );
            }
          },
        );
      },
    );
  }
}

/** @type {import("@rspack/core").Configuration} */
const config = {
  mode: 'development',
  devtool: false,
  target: 'web',
  output: {
    filename: 'bundle.js',
    chunkFilename: 'url-[id].js',
    publicPath: '/assets/',
    assetModuleFilename: '[name][ext]',
  },
  resolve: {
    byDependency: { url: { extensions: ['.js', '.txt'] } },
  },
  module: {
    rules: [
      {
        test: /target\.js$/,
        dependency: 'url',
        type: 'javascript/auto',
      },
    ],
  },
};

module.exports = [false, true].flatMap((cache) =>
  [false, undefined].map((incremental) => {
    const name = `cache-${cache}-incremental-${incremental !== false}`;
    return {
      ...config,
      name,
      cache,
      incremental,
      output: {
        ...config.output,
        filename: `bundle-${name}.js`,
        chunkFilename: `url-${name}-[id].js`,
      },
      plugins: [new CheckUrlEntryBlocksPlugin()],
    };
  }),
);
