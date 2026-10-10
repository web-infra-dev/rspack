export default {
  target: 'node',
  cache: true,
  experiments: { css: true },
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'css/module',
        generator: { exportsOnly: false },
      },
    ],
  },
  output: { cssFilename: 'style.css' },
};
