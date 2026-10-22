import path from 'node:path';
import { createRequire } from 'node:module';
import * as rspack from '@rspack/core';

const require = createRequire(import.meta.url);
const __dirname = import.meta.dirname;

/** @import { Configuration } from "@rspack/core" */
/** @typedef {"link" | "text" | "css-style-sheet" | "style"} ExportType */

const EXPORT_TYPES =
  /** @type {ExportType[]} */
  (['link', 'text', 'css-style-sheet', 'style']);

// Bun aborts in its node:vm SourceTextModule.link() and Deno hard-panics
// ("Module not found") on less-loader's `import("less")`; on both load the CJS
// less so it skips the dynamic import.
const lessLoader =
  process.versions.bun || process.versions.deno
    ? { loader: 'less-loader', options: { implementation: require('less') } }
    : 'less-loader';

/**
 * @param {ExportType} exportType css parser exportType
 * @returns {Configuration} webpack configuration
 */
const createConfig = (exportType) => ({
  name: exportType,
  target: 'web',
  mode: 'development',
  devtool: false,
  externals: {
    path: 'node-commonjs path',
  },
  module: {
    rules: [
      {
        test: /\.(css|less)$/,
        type: 'css/module',
      },
      {
        test: /\.less$/,
        use: ['./remove-source-map-url-loader.cjs', lessLoader],
      },
      {
        test: /.css$/,
        resourceQuery: /\?local-ident-name-1$/,
        type: 'css/module',
        generator: {
          // TODO do we need to support `[hash]` as  `[fullhash]` here
          localIdentName: '[name]--[local]--[fullhash]',
          localIdentHashDigest: 'base64url',
          localIdentHashDigestLength: 5,
        },
      },
      {
        test: /.css$/,
        resourceQuery: /\?local-ident-name-2$/,
        type: 'css/module',
        generator: {
          localIdentName: '-1[local]',
        },
      },
      {
        test: /.css$/,
        resourceQuery: /\?local-ident-name-3$/,
        type: 'css/module',
        generator: {
          localIdentName: '--[local]',
        },
      },
      {
        test: /.css$/,
        resourceQuery: /\?local-ident-name-4$/,
        type: 'css/module',
        generator: {
          localIdentName: '__[local]',
        },
      },
      {
        test: /.css$/,
        resourceQuery: /\?local-ident-name-5$/,
        type: 'css/module',
        generator: {
          localIdentName: '[local]--[fullhash]',
        },
      },
      {
        test: /.css$/,
        resourceQuery: /\?local-ident-name-6$/,
        type: 'css/module',
        generator: {
          localIdentName: '😀- -[local]',
        },
      },
      {
        test: /.css$/,
        resourceQuery: /\?local-ident-name-7$/,
        type: 'css/module',
        generator: {
          localIdentName: '[name]--[local]--[fullhash]',
          localIdentHashFunction: 'sha256',
          localIdentHashDigestLength: 10,
        },
      },
      {
        test: /.css$/,
        resourceQuery: /\?local-ident-name-8$/,
        type: 'css/module',
        generator: {
          localIdentName: '[name]--[local]--[fullhash]',
          localIdentHashFunction: 'xxhash64',
          localIdentHashDigestLength: 6,
        },
      },
      {
        test: /.css$/,
        resourceQuery: /\?local-ident-name-9$/,
        type: 'css/module',
        generator: {
          // Rspack accepts a template instead of webpack's callback here.
          localIdentName: 'prefix-[file][query]---[local]---[fullhash]-postfix',
        },
      },
      {
        test: /.css$/,
        resourceQuery: /\?local-ident-name-10$/,
        type: 'css/module',
        generator: {
          localIdentName: '[name]--[local]--[fullhash]',
          localIdentHashSalt: 'my-custom-salt',
        },
      },
      {
        test: /.css$/,
        resourceQuery: /\?local-ident-name-11$/,
        type: 'css/module',
        generator: {
          localIdentName: '[name]--[local]--[fullhash]',
          localIdentHashDigest: 'hex',
          localIdentHashDigestLength: 8,
        },
      },
      {
        test: /.css$/,
        resourceQuery: /\?local-ident-name-12$/,
        type: 'css/module',
        generator: {
          localIdentName: '[name]--[local]--[fullhash]',
          localIdentHashDigest: 'base64',
          localIdentHashDigestLength: 8,
        },
      },
      {
        test: /.css$/,
        resourceQuery: /\?local-ident-name-13$/,
        type: 'css/module',
        generator: {
          localIdentName: '[name]--[local]--[fullhash]',
          localIdentHashFunction: 'md4',
          localIdentHashDigest: 'base64url',
          localIdentHashDigestLength: 6,
        },
      },
      {
        test: /.css$/,
        resourceQuery: /\?local-ident-name-14$/,
        type: 'css/module',
        generator: {
          localIdentName: '[name]--[local]--[fullhash]',
          localIdentHashFunction: 'sha256',
          localIdentHashSalt: 'another-salt',
          localIdentHashDigest: 'hex',
          localIdentHashDigestLength: 12,
        },
      },
    ],
    parser: {
      css: {
        exportType,
      },
    },
  },
  resolve: {
    alias: {
      // Migration example
      '~test': path.resolve(__dirname, 'node_modules/test'),
    },
  },
  experiments: {
    css: true,
  },
  plugins: [
    new rspack.DefinePlugin({
      'process.env.EXPORT_TYPE': JSON.stringify(exportType),
    }),
  ],
});

export default EXPORT_TYPES.map(createConfig);
