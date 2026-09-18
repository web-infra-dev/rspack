import path from 'node:path';

const reactSwcOptions = {
  sourceMaps: true,
  jsc: {
    parser: {
      syntax: 'ecmascript',
      jsx: true,
    },
    transform: {
      react: {
        runtime: 'classic',
        pragma: 'React.createElement',
        pragmaFrag: 'React.Fragment',
        throwIfNamespace: true,
        useBuiltins: false,
      },
    },
  },
};

const preactSwcOptions = {
  sourceMaps: true,
  jsc: {
    parser: {
      syntax: 'ecmascript',
      jsx: true,
    },
    transform: {
      react: {
        runtime: 'classic',
        pragma: 'Preact.h',
        pragmaFrag: 'Preact.Fragment',
        throwIfNamespace: true,
        useBuiltins: false,
      },
    },
  },
};

/** @type {import("@rspack/core").Configuration} */
export default {
  context: import.meta.dirname,
  mode: 'development',
  devtool: 'source-map',
  resolve: {
    extensions: ['...', '.jsx'],
  },
  module: {
    rules: [
      {
        test: path.join(import.meta.dirname, 'swc.jsx'),
        use: [
          './loader-2.mjs',
          {
            loader: 'builtin:swc-loader',
            options: reactSwcOptions,
          },
          './loader-1.mjs',
        ],
      },
      {
        test: path.join(import.meta.dirname, 'react-refresh.jsx'),
        use: [
          './loader-2.mjs',
          'builtin:react-refresh-loader',
          {
            loader: 'builtin:swc-loader',
            options: reactSwcOptions,
          },
          './loader-1.mjs',
        ],
      },
      {
        test: path.join(import.meta.dirname, 'preact-refresh.jsx'),
        use: [
          './loader-2.mjs',
          'builtin:preact-refresh-loader',
          {
            loader: 'builtin:swc-loader',
            options: preactSwcOptions,
          },
          './loader-1.mjs',
        ],
      },
      {
        test: path.join(import.meta.dirname, 'lightning.css'),
        use: [
          './loader-2.mjs',
          {
            loader: 'builtin:lightningcss-loader',
            options: {
              minify: true,
              targets: '> 0.2%',
            },
          },
          './loader-1.mjs',
        ],
        type: 'javascript/auto',
      },
    ],
  },
};
