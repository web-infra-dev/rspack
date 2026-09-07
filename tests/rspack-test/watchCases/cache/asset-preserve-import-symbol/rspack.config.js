/** @type {import('@rspack/core').Configuration} */
module.exports = {
  cache: true,
  entry: {
    main: './index.js',
    reference: {
      import: './reference.cjs',
      dependOn: 'main',
    },
  },
  output: {
    filename: '[name].mjs',
    module: true,
    library: {
      type: 'modern-module',
    },
  },
  optimization: {
    concatenateModules: false,
    moduleIds: 'named',
  },
  module: {
    parser: {
      javascript: {
        url: 'new-url-relative',
      },
    },
    rules: [
      {
        test: /\.asset\.mjs$/,
        type: 'asset/resource',
        generator: {
          filename: 'assets/[name][ext]',
          importMode: 'preserve',
        },
      },
    ],
  },
};
