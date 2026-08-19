let compilationCount = 0;

/** @type {import('@rspack/core').Configuration} */
module.exports = {
  cache: true,
  entry: {
    main: './index.js',
    reference: {
      import: './reference.cjs',
      dependOn: 'main',
    },
    trigger: './trigger.js',
  },
  output: {
    filename: '[name].[contenthash].mjs',
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
          filename: () =>
            `${compilationCount > 1 ? 'renamed' : 'assets'}/[name][ext]`,
          importMode: 'preserve',
        },
      },
    ],
  },
  plugins: [
    (compiler) => {
      compiler.hooks.thisCompilation.tap('testcase', () => {
        compilationCount += 1;
      });
    },
  ],
};
