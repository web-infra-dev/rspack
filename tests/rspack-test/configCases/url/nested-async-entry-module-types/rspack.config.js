const { rspack } = require('@rspack/core');

class CheckUrlEntriesPlugin {
  constructor(name, scriptExtension) {
    this.name = name;
    this.scriptExtension = scriptExtension;
  }

  apply(compiler) {
    compiler.hooks.compilation.tap('CheckUrlEntriesPlugin', (compilation) => {
      compilation.hooks.finishModules.tap('CheckUrlEntriesPlugin', () => {
        const originModule = Array.from(compilation.modules).find(
          (module) => module.rawRequest === './index.js',
        );
        expect(originModule).toBeDefined();
        expect(originModule.blocks).toHaveLength(1);
        expect(
          originModule.dependencies.filter(
            (dependency) => dependency.type === 'new URL()',
          ),
        ).toHaveLength(0);
        const outer = originModule.blocks[0];
        expect(outer.blocks).toHaveLength(2);
        expect(
          outer.dependencies.filter(
            (dependency) => dependency.type === 'new URL()',
          ),
        ).toHaveLength(0);
        const inner = outer.blocks.find((block) => block.blocks.length === 1);
        const outerEntry = outer.blocks.find(
          (block) => block.blocks.length === 0,
        );
        expect(inner).toBeDefined();
        expect(
          outerEntry.dependencies.map((dependency) => dependency.request),
        ).toEqual(['./target-a.js']);
        expect(
          inner.blocks[0].dependencies.map((dependency) => dependency.request),
        ).toEqual(['./target-b.js']);
        expect(
          inner.dependencies
            .filter((dependency) => dependency.type === 'new URL()')
            .map((dependency) => dependency.request)
            .sort(),
        ).toEqual(['./target-asset.js', './target.png']);

        const assetModule = Array.from(compilation.modules).find(
          (module) => module.rawRequest === './target-asset.js',
        );
        expect(assetModule).toBeDefined();
        expect(assetModule.type).toBe('asset/resource');
      });
      compilation.hooks.processAssets.tap(
        {
          name: 'CheckUrlEntriesPlugin',
          stage: rspack.Compilation.PROCESS_ASSETS_STAGE_ADDITIONS,
        },
        () => {
          const compilationAssets = compilation.getAssets();
          const assets = compilationAssets.map((asset) => asset.name);
          const scriptAssets = assets.filter((asset) =>
            asset.endsWith(`.${this.scriptExtension}`),
          );
          expect(
            scriptAssets.filter(
              (asset) =>
                asset.startsWith(`url-${this.name}-`) &&
                !asset.includes('-inner.') &&
                !asset.includes('-outer.'),
            ),
          ).toHaveLength(2);
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
