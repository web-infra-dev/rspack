/** @type {import("@rspack/core").Configuration} */
export default {
  target: 'web',
  mode: 'development',
  module: {
    rules: [
      {
        test: /\.less$/,
        use: 'less-loader',
        type: 'css/auto',
      },
      {
        test: /\.css$/,
        type: 'css/auto',
      },
    ],
  },
  ignoreWarnings: [/ESModulesLinkingWarning: export 'class'/],
};
