const path = require('node:path');
const { experiments } = require('@rspack/core');

const { createPlugins, Layers } = experiments.rsc;
const { ServerPlugin, ClientPlugin } = createPlugins();

const ssrEntry = path.join(__dirname, 'src/framework/entry.ssr.js');
const rscEntry = path.join(__dirname, 'src/framework/entry.rsc.js');

const pageOnePath = path.join(__dirname, 'src/pages/PageOne.js');
const pageTwoPath = path.join(__dirname, 'src/pages/PageTwo.js');
const clientPaths = {
  pageOneA: path.join(__dirname, 'src/clients/PageOneClientA.js'),
  pageOneB: path.join(__dirname, 'src/clients/PageOneClientB.js'),
  pageTwo: path.join(__dirname, 'src/clients/PageTwoClient.js'),
  rootA: path.join(__dirname, 'src/clients/RootOnlyA.js'),
  rootB: path.join(__dirname, 'src/clients/RootOnlyB.js'),
  sharedAcrossPages: path.join(__dirname, 'src/clients/SharedAcrossPages.js'),
  sharedRootAndPage: path.join(__dirname, 'src/clients/SharedRootAndPage.js'),
};

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

function expectSameChunks(a, b) {
  expect(a.chunks).toEqual(b.chunks);
}

function expectDifferentChunks(a, b) {
  expect(a.chunks).not.toEqual(b.chunks);
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

          const getClient = (request) => {
            const client = mainEntry.clientManifest[request];
            expect(client).toBeDefined();
            return client;
          };

          const pageOneA = getClient(clientPaths.pageOneA);
          const pageOneB = getClient(clientPaths.pageOneB);
          const pageTwo = getClient(clientPaths.pageTwo);
          const rootA = getClient(clientPaths.rootA);
          const rootB = getClient(clientPaths.rootB);
          const sharedAcrossPages = getClient(clientPaths.sharedAcrossPages);
          const sharedRootAndPage = getClient(clientPaths.sharedRootAndPage);

          expectSameChunks(pageOneA, pageOneB);
          expectDifferentChunks(pageOneA, pageTwo);

          expectSameChunks(rootA, rootB);
          expectDifferentChunks(rootA, pageOneA);

          expectDifferentChunks(sharedAcrossPages, pageOneA);
          expectDifferentChunks(sharedAcrossPages, pageTwo);
          expectDifferentChunks(sharedRootAndPage, rootA);
          expectDifferentChunks(sharedRootAndPage, pageOneA);

          expect(
            Object.keys(mainEntry.clientManifest).filter(
              (request) => request === clientPaths.sharedAcrossPages,
            ),
          ).toHaveLength(1);
          expect(
            Object.keys(mainEntry.clientManifest).filter(
              (request) => request === clientPaths.sharedRootAndPage,
            ),
          ).toHaveLength(1);

          expect(pageOneA.cssFiles).toBeDefined();
          expect(pageOneB.cssFiles).toBeDefined();
          expect(pageTwo.cssFiles).toBeDefined();
          expect(mainEntry.entryCssFiles[pageOnePath]).toBeDefined();
          expect(mainEntry.entryCssFiles[pageTwoPath]).toBeDefined();

          expect(mainEntry.entryCssFiles[pageOnePath]).toEqual(
            pageOneA.cssFiles,
          );
          expect(mainEntry.entryCssFiles[pageOnePath]).toEqual(
            pageOneB.cssFiles,
          );
          expect(mainEntry.entryCssFiles[pageTwoPath]).toEqual(
            pageTwo.cssFiles,
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
        compiler.hooks.done.tap('AssertRscClientChunkGrouping', (stats) => {
          const { compilation } = stats;

          const pageOneCssFile = findCssAsset(
            compilation,
            'page-one-server-css',
          );
          const pageOneCss = readAsset(compilation, pageOneCssFile);
          expect(pageOneCss).toContain('page-one-client-a-css');
          expect(pageOneCss).toContain('page-one-client-b-css');
          expect(pageOneCss).not.toContain('page-two-client-css');
          expect(pageOneCss).not.toContain('shared-across-pages-client-css');

          const pageTwoCssFile = findCssAsset(
            compilation,
            'page-two-server-css',
          );
          const pageTwoCss = readAsset(compilation, pageTwoCssFile);
          expect(pageTwoCssFile).not.toBe(pageOneCssFile);
          expect(pageTwoCss).toContain('page-two-client-css');
          expect(pageTwoCss).not.toContain('page-one-client-a-css');
          expect(pageTwoCss).not.toContain('shared-across-pages-client-css');

          const rootCssFile = findCssAsset(compilation, 'root-client-a-css');
          const rootCss = readAsset(compilation, rootCssFile);
          expect(rootCssFile).not.toBe(pageOneCssFile);
          expect(rootCssFile).not.toBe(pageTwoCssFile);
          expect(rootCss).toContain('root-client-b-css');
          expect(rootCss).not.toContain('page-one-server-css');

          const sharedAcrossPagesCssFile = findCssAsset(
            compilation,
            'shared-across-pages-client-css',
          );
          expect(sharedAcrossPagesCssFile).not.toBe(pageOneCssFile);
          expect(sharedAcrossPagesCssFile).not.toBe(pageTwoCssFile);

          const sharedRootAndPageCssFile = findCssAsset(
            compilation,
            'shared-root-page-client-css',
          );
          expect(sharedRootAndPageCssFile).not.toBe(rootCssFile);
          expect(sharedRootAndPageCssFile).not.toBe(pageOneCssFile);
        });
      },
    ],
    optimization: {
      moduleIds: 'named',
      chunkIds: 'named',
    },
  },
];
