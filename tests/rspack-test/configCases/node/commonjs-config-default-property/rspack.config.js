const { rspack } = require('@rspack/core');

module.exports = {
  target: 'node',
  mode: 'development',
  plugins: [
    new rspack.DefinePlugin({
      EXPORT_KIND: JSON.stringify('commonjs-object'),
    }),
  ],
  default: {
    plugins: [
      new rspack.DefinePlugin({
        EXPORT_KIND: JSON.stringify('wrong-default'),
      }),
    ],
  },
};
