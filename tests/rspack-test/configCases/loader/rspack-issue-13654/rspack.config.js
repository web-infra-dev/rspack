const path = require('path');

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
module.exports = {
  context: __dirname,
  mode: 'development',
  devtool: 'source-map',
  resolve: {
    extensions: ['...', '.jsx'],
  },
  module: {
    rules: [
      {
        test: path.join(__dirname, 'swc.jsx'),
        use: [
          './loader-2.js',
          {
            loader: 'builtin:swc-loader',
            options: reactSwcOptions,
          },
          './loader-1.js',
        ],
      },
      {
        test: path.join(__dirname, 'react-refresh.jsx'),
        use: [
          './loader-2.js',
          'builtin:react-refresh-loader',
          {
            loader: 'builtin:swc-loader',
            options: reactSwcOptions,
          },
          './loader-1.js',
        ],
      },
      {
        test: path.join(__dirname, 'preact-refresh.jsx'),
        use: [
          './loader-2.js',
          'builtin:preact-refresh-loader',
          {
            loader: 'builtin:swc-loader',
            options: preactSwcOptions,
          },
          './loader-1.js',
        ],
      },
      {
        test: path.join(__dirname, 'lightning.css'),
        use: [
          './loader-2.js',
          {
            loader: 'builtin:lightningcss-loader',
            options: {
              minify: true,
              targets: '> 0.2%',
            },
          },
          './loader-1.js',
        ],
        type: 'javascript/auto',
      },
    ],
  },
};
