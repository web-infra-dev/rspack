const { rspack } = require('@rspack/core');

class CheckUrlEntriesPlugin {
  constructor(name, scriptExtension) {
    this.name = name;
    this.scriptExtension = scriptExtension;
  }

  apply(compiler) {
    compiler.hooks.compilation.tap('CheckUrlEntriesPlugin', (compilation) => {
      compilation.hooks.processAssets.tap(
        {
          name: 'CheckUrlEntriesPlugin',
          stage: rspack.Compilation.PROCESS_ASSETS_STAGE_ADDITIONS,
        },
        () => {
          const originModule = Array.from(compilation.modules).find(
            (module) => module.rawRequest === './index.js',
          );
          expect(originModule).toBeDefined();
          expect(originModule.blocks).toHaveLength(3);
          for (const block of originModule.blocks) {
            expect(block.dependencies).toHaveLength(1);
            expect(block.dependencies[0].type).toBe('new URL()');
          }

          const directUrlDependencies = originModule.dependencies.filter(
            (dependency) => dependency.type === 'new URL()',
          );
          expect(
            directUrlDependencies
              .map((dependency) => dependency.request)
              .sort(),
          ).toEqual(['./target-asset.js', './target.png']);

          const jsAssetModule = Array.from(compilation.modules).find(
            (module) => module.rawRequest === './target-asset.js',
          );
          expect(jsAssetModule).toBeDefined();
          expect(jsAssetModule.type).toBe('asset/resource');

          const compilationAssets = compilation.getAssets();
          const assets = compilationAssets.map((asset) => asset.name);
          const scriptAssets = assets.filter((asset) =>
            asset.endsWith(`.${this.scriptExtension}`),
          );
          expect(
            scriptAssets.filter((asset) =>
              asset.startsWith(`url-${this.name}-`),
            ),
          ).toHaveLength(2);
          const cssAssets = compilationAssets.filter((asset) =>
            asset.name.endsWith('.css'),
          );
          expect(cssAssets).toHaveLength(1);
          expect(cssAssets[0].name.startsWith(`url-${this.name}-`)).toBe(true);
          const cssSource = cssAssets[0].source.source().toString();
          expect(cssSource).toContain('.url-entry-imported');
          expect(cssSource).toContain('.url-entry-target');
          expect(assets).toContain(`target-${this.name}.png`);
          expect(assets).toContain(`target-asset-${this.name}.js`);
        },
      );
    });
  }
}

const createConfig = (name, parserUrl, outputModule = false) => {
  const scriptExtension = outputModule ? 'mjs' : 'js';
  return {
    name,
    mode: 'development',
    devtool: false,
    target: 'web',
    output: {
      module: outputModule,
      filename: `main-${name}.${scriptExtension}`,
      chunkFilename: `url-${name}-[id].${scriptExtension}`,
      cssChunkFilename: `url-${name}-[id].css`,
      assetModuleFilename: `[name]-${name}[ext]`,
      publicPath: name === 'relative' ? 'assets/' : '/assets/',
    },
    module: {
      parser: {
        javascript: {
          url: parserUrl,
        },
      },
      rules: [
        {
          test: /target-[ab]\.js$/,
          dependency: 'url',
          type: 'javascript/auto',
        },
        {
          test: /target\.css$/,
          dependency: 'url',
          type: 'css',
        },
        {
          test: /target-imported\.css$/,
          type: 'css',
        },
        {
          test: /target\.png$/,
          dependency: 'url',
          type: 'asset/resource',
        },
        {
          test: /target-asset\.js$/,
        },
      ],
    },
    plugins: [
      new rspack.DefinePlugin({
        URL_MODE: JSON.stringify(name),
      }),
      new CheckUrlEntriesPlugin(name, scriptExtension),
    ],
  };
};

/** @type {import("@rspack/core").Configuration[]} */
module.exports = [
  createConfig('default', true),
  createConfig('relative', 'relative'),
  createConfig('new-url-relative', 'new-url-relative', true),
];
