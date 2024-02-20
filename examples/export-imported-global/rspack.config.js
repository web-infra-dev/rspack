// rspack.config.js
module.exports =  {
  entry: {
    main: "./index.js",
  },
  optimization: {
    concatenateModules: false,
    minimize: false,
  },
  module: {
    rules: [
      {
        test: /\.css$/,
        type: "css/module",
      },
    ],
  },
  experiments: {
    css: true,
    rspackFuture: {
      newTreeshaking: true,
    },
  },
};
