class CheckUrlTargetPlugin {
  apply(compiler) {
    let buildIndex = 0;
    let issuerBuilds = 0;
    compiler.hooks.compilation.tap('CheckUrlTargetPlugin', (compilation) => {
      const currentBuild = buildIndex++;
      compilation.hooks.buildModule.tap('CheckUrlTargetPlugin', (module) => {
        if (module.rawRequest === './index.js') issuerBuilds++;
      });
      compilation.hooks.finishModules.tap('CheckUrlTargetPlugin', (modules) => {
        expect(issuerBuilds).toBe(currentBuild + 1);
        const issuer = [...modules].find(
          (module) => module.rawRequest === './index.js',
        );
        const dependency = [
          ...issuer.dependencies,
          ...issuer.blocks.flatMap((block) => block.dependencies),
        ].find((dependency) => dependency.type === 'new URL()');
        expect(compilation.moduleGraph.getModule(dependency).type).toBe(
          currentBuild % 2 === 0 ? 'javascript/auto' : 'asset/resource',
        );
        expect(issuer.blocks).toHaveLength(currentBuild % 2 === 0 ? 1 : 0);
        if (currentBuild % 2 === 0) {
          expect(issuer.blocks[0].dependencies).toHaveLength(1);
        } else {
          expect(issuer.dependencies).toContain(dependency);
        }
      });
      compilation.hooks.processAssets.tap('CheckUrlTargetPlugin', () => {
        if (currentBuild % 2 === 0) return;
        const chunk = compilation.entrypoints.get('main').getEntrypointChunk();
        const modules = compilation.chunkGraph.getChunkModulesIterable(chunk);
        expect(modules.some((module) => module.type === 'asset/resource')).toBe(
          true,
        );
        expect(compilation.getAsset('target.txt')).toBeDefined();
        expect(
          compilation.getAsset('target.txt').source.source().toString().trim(),
        ).toBe(
          currentBuild === 3 ? 'updated asset content' : 'URL target asset',
        );
      });
    });
  }
}

/** @type {import('@rspack/core').Configuration} */
const config = {
  mode: 'development',
  devtool: false,
  target: 'web',
  cache: true,
  output: {
    filename: 'bundle.js',
    chunkFilename: 'url-[id].js',
    assetModuleFilename: '[name][ext]',
    publicPath: '/assets/',
  },
  resolve: {
    byDependency: {
      url: { extensions: ['.js', '.txt'] },
    },
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
  plugins: [new CheckUrlTargetPlugin()],
};

module.exports = [false, true].map((incremental) => ({
  ...config,
  incremental,
  output: {
    ...config.output,
    filename: `bundle-${incremental}.js`,
    chunkFilename: `url-${incremental}-[id].js`,
  },
  plugins: [new CheckUrlTargetPlugin()],
}));
