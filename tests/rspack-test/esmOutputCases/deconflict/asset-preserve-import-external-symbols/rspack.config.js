module.exports = {
  entry: {
    main: './index.js',
    reference: {
      import: './reference.cjs',
      dependOn: 'main',
    },
  },
  externals: {
    'virtual-fs': 'module fs',
    'virtual-url': 'module node:url',
  },
  node: {
    __dirname: 'node-module',
  },
  output: {
    environment: {
      importMetaDirnameAndFilename: false,
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
        test: /(?:fileURLToPath|readFile)\.mjs$/,
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
