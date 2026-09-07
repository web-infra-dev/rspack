const { CopyRspackPlugin } = require('@rspack/core');

module.exports = {
  entry: './index.js',
  target: 'node',
  plugins: [
    new CopyRspackPlugin({
      patterns: [
        {
          from: 'assets/glob/*/*.txt',
          to: 'copied',
          toType: 'dir',
        },
        {
          from: 'assets/simple-template',
          to: 'template/[name][ext]',
        },
        {
          from: 'assets/template',
          to: 'template/[path][name][ext]',
        },
      ],
    }),
  ],
};
