export default {
  resolve: { alias: { '@theme': './theme', optional: './optional.less' } },
  module: {
    rules: [
      { test: /\.less$/, type: 'asset/source', use: 'builtin:less-loader' },
    ],
  },
};
