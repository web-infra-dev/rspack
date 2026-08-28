module.exports = {
  module: {
    rules: [
      {
        test: /resource\.js$/,
        use: [{ loader: './loader.js', parallel: true, options: {} }],
      },
    ],
  },
};
