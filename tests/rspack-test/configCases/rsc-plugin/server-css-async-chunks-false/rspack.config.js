const path = require('node:path');
const { experiments } = require('@rspack/core');

const { createPlugins, Layers } = experiments.rsc;
const { ServerPlugin, ClientPlugin } = createPlugins();

const ssrEntry = path.join(__dirname, 'src/framework/entry.ssr.js');
const rscEntry = path.join(__dirname, 'src/framework/entry.rsc.js');
const clientEntry = path.join(__dirname, 'src/framework/entry.client.js');
const rootPath = path.join(__dirname, 'src/Root.js');
const pageAPath = path.join(__dirname, 'src/pages/PageA.js');
const pageANestedPath = path.join(__dirname, 'src/pages/PageANested.js');
const pageBPath = path.join(__dirname, 'src/pages/PageB.js');

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
          expect(Object.keys(mainEntry.entryCssFiles).sort()).toEqual(
            [pageAPath, pageANestedPath, pageBPath].sort(),
          );

          expect(mainEntry.entryCssFiles[pageAPath]).toBeDefined();
          expect(mainEntry.entryCssFiles[pageAPath].length).toBe(1);
          expect(mainEntry.entryCssFiles[pageAPath][0]).toMatch(/\.css$/);

          expect(mainEntry.entryCssFiles[pageANestedPath]).toBeDefined();
          expect(mainEntry.entryCssFiles[pageANestedPath].length).toBe(1);
          expect(mainEntry.entryCssFiles[pageANestedPath][0]).toMatch(/\.css$/);

          expect(mainEntry.entryCssFiles[pageBPath]).toBeDefined();
          expect(mainEntry.entryCssFiles[pageBPath].length).toBe(1);
          expect(mainEntry.entryCssFiles[pageBPath][0]).toMatch(/\.css$/);

          expect(mainEntry.entryCssFiles[pageAPath][0]).toBe(
            mainEntry.entryCssFiles[pageANestedPath][0],
          );
          expect(mainEntry.entryCssFiles[pageAPath][0]).toBe(
            mainEntry.entryCssFiles[pageBPath][0],
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
        import: clientEntry,
        asyncChunks: false,
      },
    },
    resolve: {
      extensions: ['...', '.ts', '.tsx', '.jsx'],
    },
    module: {
      rules: [cssRule, swcLoaderRule],
    },
    output: {
      filename: 'client/[name].js',
    },
    plugins: [
      new ClientPlugin(),
      (compiler) => {
        compiler.hooks.done.tap('AssertInlinedServerCss', (stats) => {
          const { compilation } = stats;
          const entrypoint = compilation.entrypoints.get('main');
          const entryCssFiles = entrypoint
            .getFiles()
            .filter((file) => file.endsWith('.css'));
          expect(entryCssFiles.length).toBeGreaterThan(0);

          const rootCssFile = findCssAsset(
            compilation,
            'root-async-chunks-false-css',
          );
          const css = readAsset(compilation, rootCssFile);
          expect(entryCssFiles).toContain(rootCssFile);
          expect(
            compilation.getAssets().filter(({ name }) => name.endsWith('.css'))
              .length,
          ).toBe(1);

          for (const marker of [
            'root-async-chunks-false-css',
            'page-a-async-chunks-false-css',
            'page-a-child-async-chunks-false-css',
            'server-entry-shared-async-chunks-false-css',
            'page-a-nested-async-chunks-false-css',
            'page-b-async-chunks-false-css',
            'page-b-child-async-chunks-false-css',
          ]) {
            expect(css).toContain(marker);
            expect(findCssAsset(compilation, marker)).toBe(rootCssFile);
          }
        });
      },
    ],
    optimization: {
      moduleIds: 'named',
      chunkIds: 'named',
    },
  },
];
