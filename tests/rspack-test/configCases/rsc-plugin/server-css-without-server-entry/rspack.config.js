const path = require('node:path');
const { experiments } = require('@rspack/core');

const { createPlugins, Layers } = experiments.rsc;
const { ServerPlugin, ClientPlugin } = createPlugins();

const ssrEntry = path.join(__dirname, 'src/framework/entry.ssr.js');
const rscEntry = path.join(__dirname, 'src/framework/entry.rsc.js');

const swcLoaderRule = {
  test: /\.jsx?$/,
  use: [
    {
      loader: 'builtin:swc-loader',
      options: {
        jsc: {
          parser: {
            syntax: 'ecmascript',
            jsx: true,
          },
          transform: {
            react: {
              runtime: 'automatic',
            },
          },
        },
        rspackExperiments: {
          reactServerComponents: true,
        },
      },
    },
  ],
};

const cssRule = {
  test: /\.css$/,
  type: 'css/auto',
};

module.exports = [
  {
    mode: 'production',
    target: 'node',
    entry: {
      main: {
        import: ssrEntry,
      },
    },
    resolve: {
      extensions: ['...', '.ts', '.tsx', '.jsx'],
    },
    module: {
      rules: [
        cssRule,
        swcLoaderRule,
        {
          resource: ssrEntry,
          layer: Layers.ssr,
        },
        {
          resource: rscEntry,
          layer: Layers.rsc,
          resolve: {
            conditionNames: ['react-server', '...'],
          },
        },
        {
          issuerLayer: Layers.rsc,
          resolve: {
            conditionNames: ['react-server', '...'],
          },
        },
      ],
    },
    plugins: [
      new ServerPlugin({
        onManifest(manifest) {
          const mainEntry = manifest.main;
          expect(mainEntry).toBeDefined();
          expect(mainEntry.entryJsFiles.length).toBe(1);

          const appPath = path.join(__dirname, 'src/App.js');
          expect(Object.keys(mainEntry.entryCssFiles)).toEqual([]);
          expect(mainEntry.entryCssFiles[appPath]).toBeUndefined();
        },
      }),
    ],
    optimization: {
      moduleIds: 'named',
      chunkIds: 'named',
    },
  },
  {
    mode: 'production',
    target: 'web',
    entry: {
      main: {
        import: './src/framework/entry.client.js',
      },
    },
    resolve: {
      extensions: ['...', '.ts', '.tsx', '.jsx'],
    },
    module: {
      rules: [cssRule, swcLoaderRule],
    },
    plugins: [
      new ClientPlugin(),
      (compiler) => {
        compiler.hooks.done.tap('AssertServerCssInClientOutput', (stats) => {
          const cssAssets = stats.compilation
            .getAssets()
            .filter((asset) => asset.name.endsWith('.css'));
          expect(cssAssets.length).toBeGreaterThan(0);

          const hasServerCss = cssAssets.some((asset) =>
            asset.source.source().toString().includes('seagreen'),
          );
          expect(hasServerCss).toBe(true);
        });
      },
    ],
    optimization: {
      moduleIds: 'named',
      chunkIds: 'named',
    },
  },
];
