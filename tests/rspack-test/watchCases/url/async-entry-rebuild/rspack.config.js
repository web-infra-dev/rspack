const { rspack } = require('@rspack/core');

class CheckUrlEntryBlocksPlugin {
  apply(compiler) {
    let buildIndex = 0;
    compiler.hooks.compilation.tap(
      'CheckUrlEntryBlocksPlugin',
      (compilation) => {
        const currentBuild = buildIndex++;
        const hasCssEntry = currentBuild !== 2;
        compilation.hooks.finishModules.tap(
          {
            name: 'CheckUrlEntryBlocksPlugin',
            stage: -100,
          },
          () => {
            const originModule = Array.from(compilation.modules).find(
              (module) => module.rawRequest === './index.js',
            );
            expect(originModule).toBeDefined();
            expect(originModule.blocks).toHaveLength(hasCssEntry ? 2 : 1);
            for (const block of originModule.blocks) {
              expect(block.dependencies).toHaveLength(1);
              expect(block.dependencies[0].type).toBe('new URL()');
            }
            expect(
              originModule.dependencies.filter(
                (dependency) => dependency.type === 'new URL()',
              ),
            ).toHaveLength(0);
          },
        );
        compilation.hooks.processAssets.tap(
          {
            name: 'CheckUrlEntryBlocksPlugin',
            stage: rspack.Compilation.PROCESS_ASSETS_STAGE_ADDITIONS,
          },
          () => {
            const assets = compilation.getAssets().map((asset) => asset.name);
            expect(
              assets.filter((asset) => asset.endsWith('.js')),
            ).toHaveLength(2);
            expect(
              assets.filter((asset) => asset.endsWith('.css')),
            ).toHaveLength(hasCssEntry ? 1 : 0);
            const jsAsset = compilation
              .getAssets()
              .find(
                (asset) =>
                  asset.name.startsWith('url-') && asset.name.endsWith('.js'),
              );
            expect(jsAsset.source.source().toString()).toContain(
              currentBuild === 0 ? 'initial' : 'updated',
            );
            if (hasCssEntry) {
              const cssAsset = compilation
                .getAssets()
                .find((asset) => asset.name.endsWith('.css'));
              expect(cssAsset.source.source().toString()).toContain(
                currentBuild === 4
                  ? '.url-entry-rebuilt'
                  : '.url-entry-rebuild-target',
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
};

// Exercise both a cached unchanged origin and module-graph rollback independently.
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
        cssChunkFilename: `url-${name}-[id].css`,
      },
      plugins: [new CheckUrlEntryBlocksPlugin()],
    };
  }),
);
