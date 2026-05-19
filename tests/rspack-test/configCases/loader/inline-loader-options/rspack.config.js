module.exports = {
  module: {
    rules: [
      {
        loader: 'builtin:swc-loader',
        options: {
          detectSyntax: 'auto',
        },
      },
    ],
  },
};
