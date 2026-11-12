import { defineConfig, definePlugin } from '@rspack/cli';
import {
  CssExtractRspackPlugin,
  type OptimizationSplitChunksOptions,
} from '@rspack/core';

// The loaders make shared JS 60KB and each stylesheet about 24KB. At a 50KB
// reduction threshold, CSS fits with three consumers but fails with two.
const cases: {
  name: string;
  minSizeReduction: OptimizationSplitChunksOptions['minSizeReduction'];
  minSize?: OptimizationSplitChunksOptions['minSize'];
  entry?: string;
  update?: boolean;
  jsCopies: number;
  cssChunks: number;
}[] = [
  {
    name: 'three-consumers',
    entry: './three',
    minSizeReduction: 50_000,
    jsCopies: 1,
    cssChunks: 1,
  },
  {
    name: 'css-below-reduction',
    minSizeReduction: 50_000,
    jsCopies: 1,
    cssChunks: 2,
  },
  {
    name: 'empty-min-size',
    minSize: {},
    minSizeReduction: 50_000,
    jsCopies: 1,
    cssChunks: 2,
  },
  {
    name: 'js-below-reduction',
    minSizeReduction: { javascript: 130_000, 'css/mini-extract': 30_000 },
    jsCopies: 2,
    cssChunks: 1,
  },
  {
    name: 'both-below-reduction',
    minSizeReduction: 130_000,
    jsCopies: 2,
    cssChunks: 2,
  },
  {
    name: 'both-above-reduction',
    minSizeReduction: 1_000,
    jsCopies: 1,
    cssChunks: 1,
  },
  {
    name: 'zero-css-reduction',
    minSizeReduction: { javascript: 50_000, 'css/mini-extract': 0 },
    jsCopies: 1,
    cssChunks: 1,
  },
  // Extracting shared.css first leaves other.css below the reduction threshold.
  // Revalidating the remaining mixed candidate must still keep shared JS.
  {
    name: 'candidate-update',
    entry: './update',
    update: true,
    minSizeReduction: 50_000,
    jsCopies: 1,
    cssChunks: 3,
  },
];

export default cases.map(
  ({
    name,
    minSizeReduction,
    minSize = 1_000,
    entry = './index',
    update,
    jsCopies,
    cssChunks,
  }) =>
    defineConfig({
      name,
      mode: 'production',
      target: 'node',
      entry,
      output: {
        filename: `${name}.js`,
        chunkFilename: `${name}-[id].js`,
      },
      module: {
        rules: [
          {
            test: /\.css$/,
            type: 'javascript/auto',
            use: [
              CssExtractRspackPlugin.loader,
              'css-loader',
              '../min-remaining-size-css/repeat-loader',
            ],
          },
          {
            test: /[\\/]shared\.js$/,
            loader: '../min-remaining-size/size-loader',
            options: { size: 60_000 },
          },
        ],
      },
      optimization: {
        minimize: false,
        concatenateModules: false,
        inlineExports: false,
        splitChunks: {
          chunks: 'all',
          minSize,
          minSizeReduction,
          defaultSizeTypes: ['javascript', 'css/mini-extract'],
          maxAsyncRequests: Infinity,
          maxInitialRequests: Infinity,
          cacheGroups: {
            defaultVendors: false,
            default: { minChunks: 2, reuseExistingChunk: true },
            ...(update
              ? {
                  firstCss: {
                    test: /shared\.css/,
                    name: 'first-css',
                    priority: 10,
                    enforce: true,
                    minSize: 0,
                    minSizeReduction: 0,
                  },
                }
              : {}),
          },
        },
      },
      plugins: [
        new CssExtractRspackPlugin({ filename: `${name}-[id].css` }),
        definePlugin(function () {
          this.hooks.compilation.tap(name, (compilation) => {
            compilation.hooks.afterSeal.tap(name, () => {
              const chunks = [...compilation.chunks];
              const modules = (chunk: (typeof chunks)[number]) => [
                ...compilation.chunkGraph.getChunkModulesIterable(chunk),
              ];
              const sharedJs = chunks.filter((chunk) =>
                modules(chunk).some((module) =>
                  module
                    .nameForCondition()
                    ?.replaceAll('\\', '/')
                    .endsWith('/shared.js'),
                ),
              );
              const css = chunks.filter((chunk) =>
                modules(chunk).some(
                  (module) => module.type === 'css/mini-extract',
                ),
              );
              expect({ name, copies: sharedJs.length }).toEqual({
                name,
                copies: jsCopies,
              });
              expect({ name, chunks: css.length }).toEqual({
                name,
                chunks: cssChunks,
              });
            });
          });
        }),
      ],
    }),
);
