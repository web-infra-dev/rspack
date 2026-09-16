export default {
  output: {
    uniqueName: 'app',
  },
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'css/auto',
      },
    ],
  },
};
