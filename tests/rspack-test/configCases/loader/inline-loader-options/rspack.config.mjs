export default {
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
