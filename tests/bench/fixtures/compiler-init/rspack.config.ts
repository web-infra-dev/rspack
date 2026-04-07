import path from 'node:path';
import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';
import { ReactRefreshRspackPlugin } from '@rspack/plugin-react-refresh';

const BROWSERS_LIST = [
  'chrome >= 51',
  'edge >= 15',
  'firefox >= 54',
  'safari >= 10',
  'ios_saf >= 10',
];
const prod = process.env.NODE_ENV === 'production';
const POSTCSS_LOADER = {
  loader: 'postcss-loader',
  options: {
    postcssOptions: {
      plugins: [
        {
          browsers: BROWSERS_LIST,
          options: {
            flexbox: 'no-2009',
          },
          postcssPlugin: 'autoprefixer',
        },
      ],
      config: false,
    },
  },
};

export default defineConfig({
  mode: 'development',
  context: __dirname,
  target: 'web',
  // Keep entry empty so `compiler.run()` still exercises compiler creation and
  // config normalization, but avoids pulling real module graphs into the
  // benchmark. This keeps the signal focused on initialization-heavy work.
  entry: {},
  experiments: {
    asyncWebAssembly: true,
    css: true,
  },
  cache: false,
  output: {
    path: path.join(__dirname, 'dist'),
    filename: 'static/js/[name].[contenthash:8].js',
    chunkFilename: 'static/js/async/[name].[contenthash:8].js',
    publicPath: '/',
    hashFunction: 'xxhash64',
    webassemblyModuleFilename: 'static/wasm/[hash].module.wasm',
    cssFilename: 'static/css/[name].[contenthash:8].css',
    cssChunkFilename: 'static/css/async/[name].[contenthash:8].css',
  },
  resolve: {
    extensions: ['.ts', '.tsx', '.js', '.jsx', '.mjs', '.json'],
    tsConfig: path.join(__dirname, 'tsconfig.json'),
  },
  module: {
    parser: {
      'css/module': {
        namedExports: false,
      },
    },
    rules: [
      {
        test: /\.(?:png|jpg|jpeg|pjpeg|pjp|gif|bmp|webp|ico|apng|avif|tif|tiff|jfif)$/i,
        oneOf: [
          {
            type: 'asset/resource',
            resourceQuery: /(__inline=false|url)/,
            generator: { filename: 'static/image/[name].[contenthash:8][ext]' },
          },
          { type: 'asset/inline', resourceQuery: /inline/ },
          {
            type: 'asset',
            parser: { dataUrlCondition: { maxSize: 10000 } },
            generator: { filename: 'static/image/[name].[contenthash:8][ext]' },
          },
        ],
      },
      {
        test: /\.(?:mp4|webm|ogg|mov|mp3|wav|flac|aac|m4a|opus)$/i,
        oneOf: [
          {
            type: 'asset/resource',
            resourceQuery: /(__inline=false|url)/,
            generator: { filename: 'static/media/[name].[contenthash:8][ext]' },
          },
          { type: 'asset/inline', resourceQuery: /inline/ },
          {
            type: 'asset',
            parser: { dataUrlCondition: { maxSize: 10000 } },
            generator: { filename: 'static/media/[name].[contenthash:8][ext]' },
          },
        ],
      },
      {
        test: /\.(?:woff|woff2|eot|ttf|otf|ttc)$/i,
        oneOf: [
          {
            type: 'asset/resource',
            resourceQuery: /(__inline=false|url)/,
            generator: { filename: 'static/font/[name].[contenthash:8][ext]' },
          },
          { type: 'asset/inline', resourceQuery: /inline/ },
          {
            type: 'asset',
            parser: { dataUrlCondition: { maxSize: 10000 } },
            generator: { filename: 'static/font/[name].[contenthash:8][ext]' },
          },
        ],
      },
      {
        test: /\.wasm$/,
        dependency: 'url',
        type: 'asset/resource',
        generator: { filename: 'static/wasm/[hash].module.wasm' },
      },
      {
        test: /\.css$/,
        oneOf: [
          {
            sideEffects: true,
            use: [POSTCSS_LOADER],
            resolve: { preferRelative: true },
            test: /\.module\.\w+$/i,
            type: 'css/module',
          },
          {
            sideEffects: true,
            use: [POSTCSS_LOADER],
            resolve: { preferRelative: true },
            type: 'css',
          },
        ],
      },
      {
        test: /\.less$/,
        oneOf: [
          {
            sideEffects: true,
            use: [
              POSTCSS_LOADER,
              {
                loader: 'less-loader',
                options: {
                  lessOptions: { javascriptEnabled: true },
                },
              },
            ],
            resolve: { preferRelative: true },
            test: /\.module\.\w+$/i,
            type: 'css/module',
          },
          {
            sideEffects: true,
            use: [
              POSTCSS_LOADER,
              {
                loader: 'less-loader',
                options: {
                  lessOptions: { javascriptEnabled: true },
                },
              },
            ],
            resolve: { preferRelative: true },
            type: 'css',
          },
        ],
      },
      {
        test: /\.(j|t)s(x)?$/,
        loader: 'builtin:swc-loader',
        exclude: [/[\\/]node_modules[\\/]/],
        options: {
          sourceMaps: true,
          jsc: {
            parser: {
              syntax: 'typescript',
              tsx: true,
            },
            transform: {
              react: {
                runtime: 'automatic',
                development: !prod,
                refresh: !prod,
              },
            },
            externalHelpers: true,
          },
          env: {
            targets: 'Chrome >= 48',
          },
        },
      },
      {
        test: /\.svg$/,
        oneOf: [
          {
            type: 'asset/resource',
            resourceQuery: /(__inline=false|url)/,
            generator: { filename: 'static/svg/[name].[contenthash:8].svg' },
            issuer: {
              not: [/\.(?:js|mjs|cjs|jsx)$/, /\.(?:ts|mts|cts|tsx)$/],
            },
          },
          {
            type: 'asset/inline',
            resourceQuery: /inline/,
            issuer: {
              not: [/\.(?:js|mjs|cjs|jsx)$/, /\.(?:ts|mts|cts|tsx)$/],
            },
          },
          {
            type: 'asset',
            parser: { dataUrlCondition: { maxSize: 10000 } },
            generator: { filename: 'static/svg/[name].[contenthash:8].svg' },
            issuer: {
              not: [/\.(?:js|mjs|cjs|jsx)$/, /\.(?:ts|mts|cts|tsx)$/],
            },
          },
          { type: 'asset/inline', resourceQuery: /inline/ },
          {
            type: 'asset/resource',
            resourceQuery: /url/,
            generator: { filename: 'static/svg/[name].[contenthash:8].svg' },
          },
          {
            type: 'javascript/auto',
            use: [
              {
                loader: 'builtin:swc-loader',
                options: {
                  jsc: {
                    externalHelpers: true,
                    parser: {
                      tsx: true,
                      syntax: 'typescript',
                      decorators: true,
                    },
                    preserveAllComments: true,
                    transform: {
                      legacyDecorator: true,
                      decoratorMetadata: true,
                      react: {
                        development: true,
                        refresh: true,
                        runtime: 'automatic',
                      },
                    },
                  },
                  env: {
                    targets: BROWSERS_LIST,
                    mode: 'usage',
                    coreJs: '3.32',
                    shippedProposals: true,
                  },
                },
              },
              {
                loader: '@svgr/webpack',
                options: {
                  svgo: true,
                  svgoConfig: {
                    plugins: [
                      {
                        name: 'preset-default',
                        params: { overrides: { removeViewBox: false } },
                      },
                      'prefixIds',
                    ],
                  },
                },
              },
              {
                loader: 'url-loader',
                options: {
                  limit: 10000,
                  name: 'static/svg/[name].[contenthash:8].svg',
                },
              },
            ],
          },
        ],
      },
    ],
  },
  optimization: {
    splitChunks: {
      chunks: 'all',
      cacheGroups: {
        'lib-lodash': {
          test: /[\\/]node_modules[\\/](lodash|lodash-es)[\\/]/,
          priority: 0,
          name: 'lib-lodash',
          reuseExistingChunk: true,
        },
        'lib-axios': {
          test: /[\\/]node_modules[\\/](axios|axios-.+)[\\/]/,
          priority: 0,
          name: 'lib-axios',
          reuseExistingChunk: true,
        },
        'lib-polyfill': {
          test: /[\\/]node_modules[\\/](tslib|core-js|@babel\/runtime|@swc\/helpers)[\\/]/,
          priority: 0,
          name: 'lib-polyfill',
          reuseExistingChunk: true,
        },
        'lib-react': {
          test: /[\\/]node_modules[\\/](react|react-dom|scheduler)[\\/]/,
          priority: 0,
          name: 'lib-react',
          reuseExistingChunk: true,
        },
        'lib-router': {
          test: /[\\/]node_modules[\\/](react-router|react-router-dom|history|@remix-run[\\/]router)[\\/]/,
          priority: 0,
          name: 'lib-router',
          reuseExistingChunk: true,
        },
      },
    },
  },
  plugins: [
    new rspack.HtmlRspackPlugin({
      templateContent:
        '<!doctype html><html><head></head><body><div id="root"></div></body></html>',
    }),
    !prod ? new ReactRefreshRspackPlugin() : null,
    {
      apply(compiler) {
        compiler.hooks.done.tap('CompilerInitBenchPlugin', () => {});
      },
    },
  ].filter(Boolean),
  devServer: {
    static: {
      directory: path.join(__dirname, 'dist'),
    },
    historyApiFallback: true,
  },
});
