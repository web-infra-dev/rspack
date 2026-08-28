const { CopyRspackPlugin } = require('@rspack/core');

module.exports = {
  entry: './index.js',
  target: 'node',
  plugins: [
    new CopyRspackPlugin({
      patterns: [
        {
          from: 'src/{foo,bar}.*.yml',
          to: 'wildcard/[name][ext]',
        },
        {
          from: 'src/{alpha,beta}.txt',
          to: 'literal/[name][ext]',
        },
        {
          from: 'src/{one,two}/**/*.txt',
          to: 'nested/[path][name][ext]',
        },
      ],
    }),
  ],
  output: {
    clean: true,
  },
};
