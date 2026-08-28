module.exports = {
  module: {
    rules: [
      {
        test: /module\.js$/,
        use: [
          { loader: './loader.js', parallel: true, options: {} },
          {
            loader: 'builtin:swc-loader',
            options: { jsc: { parser: { syntax: 'ecmascript' } } },
          },
        ],
      },
    ],
  },
};
