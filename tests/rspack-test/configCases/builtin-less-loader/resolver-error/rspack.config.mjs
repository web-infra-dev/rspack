export default {
  module: {
    rules: [
      { test: /\.less$/, type: 'asset/source', use: 'builtin:less-loader' },
    ],
  },
};
