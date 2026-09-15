module.exports = {
  entry: {
    main: './index.js',
    reference: {
      import: './reference.cjs',
      dependOn: 'main',
    },
  },
  module: {
    parser: {
      javascript: {
        url: 'new-url-relative',
      },
    },
    rules: [
      {
        test: /(?:__webpack_require__|rspackRequire)\.mjs$/,
        type: 'asset/resource',
        generator: {
          filename: 'assets/[name][ext]',
          importMode: 'preserve',
        },
      },
    ],
  },
  optimization: {
    runtimeChunk: false,
  },
};
