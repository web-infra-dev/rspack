const TsconfigPathsPlugin = require("tsconfig-paths-webpack-plugin");

module.exports = {
  entry: {
    main: "./src/index.ts"
  },
  module: {
    rules: [
      {
        test: /\.tsx?$/,
        type: "javascript/auto"
      }
    ]
  },
  optimization: {
    minimize: false,
    moduleIds: "named"
  },
  resolve: {
    extensions: [".tsx", ".ts", "..."],
    plugins: [
      new TsconfigPathsPlugin({
        logLevel: "info",
        extensions: [".tsx", ".ts", ".js"],
        baseUrl: "./",
        logInfoToStdOut: true,
        configFile: "./tsconfig.json"
        // uncomment this see different generated files
        // references: ["../component/tsconfig.json"],
      })
    ]
  }
};
