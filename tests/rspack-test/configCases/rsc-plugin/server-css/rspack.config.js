const path = require('node:path');
const { experiments } = require('@rspack/core');

const { createPlugins, Layers } = experiments.rsc;
const { ServerPlugin, ClientPlugin } = createPlugins();

const ssrEntry = path.join(__dirname, 'src/framework/entry.ssr.js');
const rscEntry = path.join(__dirname, 'src/framework/entry.rsc.js');
const rootPath = path.join(__dirname, 'src/Root.js');
const page1Path = path.join(__dirname, 'src/pages/Page1.js');
const page2Path = path.join(__dirname, 'src/pages/Page2.js');

const swcLoaderRule = {
  test: /\.jsx?$/,
  use: [
    {
      loader: 'builtin:swc-loader',
      options: {
        detectSyntax: 'auto',
        jsc: {
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

function readAsset(compilation, file) {
  return compilation.getAsset(file).source.source().toString();
}

function findCssAsset(compilation, marker) {
  const cssAsset = compilation
    .getAssets()
    .filter(({ name }) => name.endsWith('.css'))
    .find(({ source }) => source.source().toString().includes(marker));
  expect(cssAsset).toBeDefined();
  return cssAsset.name;
}

module.exports = [
  {
    mode: 'development',
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

          expect(mainEntry.entryCssFiles[rootPath]).toBeUndefined();

          expect(mainEntry.entryCssFiles[page1Path]).toBeDefined();
          expect(mainEntry.entryCssFiles[page1Path].length).toBe(1);
          expect(mainEntry.entryCssFiles[page1Path][0]).toMatch(/\.css$/);

          expect(mainEntry.entryCssFiles[page2Path]).toBeDefined();
          expect(mainEntry.entryCssFiles[page2Path].length).toBe(1);
          expect(mainEntry.entryCssFiles[page2Path][0]).toMatch(/\.css$/);

          expect(mainEntry.entryCssFiles[page1Path][0]).not.toBe(
            mainEntry.entryCssFiles[page2Path][0],
          );
        },
      }),
    ],
    optimization: {
      moduleIds: 'named',
      chunkIds: 'named',
    },
  },
  {
    mode: 'development',
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
        compiler.hooks.done.tap('AssertServerCssChunking', (stats) => {
          const { compilation } = stats;
          const entrypoint = compilation.entrypoints.get('main');
          const entryCssFiles = entrypoint
            .getFiles()
            .filter((file) => file.endsWith('.css'));
          expect(entryCssFiles.length).toBeGreaterThan(0);

          const rootCssFile = findCssAsset(compilation, 'root-server-css');
          expect(entryCssFiles).toContain(rootCssFile);
          expect(readAsset(compilation, rootCssFile)).toContain(
            'root-server-css',
          );

          const page1CssFile = findCssAsset(compilation, 'page-one-css');
          const page1Css = readAsset(compilation, page1CssFile);
          expect(page1CssFile).not.toBe(rootCssFile);
          expect(entryCssFiles).not.toContain(page1CssFile);
          expect(page1Css).toContain('page-one-css');
          expect(page1Css).toContain('page-one-child-css');
          expect(page1Css).not.toContain('page-two-css');
          expect(page1Css).not.toContain('page-two-child-css');

          const page2CssFile = findCssAsset(compilation, 'page-two-css');
          const page2Css = readAsset(compilation, page2CssFile);
          expect(page2CssFile).not.toBe(rootCssFile);
          expect(page2CssFile).not.toBe(page1CssFile);
          expect(entryCssFiles).not.toContain(page2CssFile);
          expect(page2Css).toContain('page-two-css');
          expect(page2Css).toContain('page-two-child-css');
          expect(page2Css).not.toContain('page-one-css');
          expect(page2Css).not.toContain('page-one-child-css');
        });
      },
    ],
    optimization: {
      moduleIds: 'named',
      chunkIds: 'named',
    },
  },
];
