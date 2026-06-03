/** @type {import("@rspack/core").Configuration} */
module.exports = {
  target: 'web',
  node: {
    __dirname: false,
    __filename: false,
  },
  module: {
    parser: {
      'css/module': {
        animation: true,
        container: true,
        customIdents: true,
        dashedIdents: true,
        function: true,
        grid: true,
      },
    },
    generator: {
      'css/module': {
        localIdentName: '[local]--local',
      },
    },
    rules: [
      {
        test: /\.module\.css$/,
        type: 'css/module',
      },
    ],
  },
};
