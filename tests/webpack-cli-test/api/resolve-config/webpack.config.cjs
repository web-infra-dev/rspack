module.exports = [
  {
    output: {
      filename: "./dist-amd.js",
      libraryTarget: "amd",
    },
    entry: "./a.js",
    name: "amd",
    mode: "development",
    devtool: "eval-cheap-module-source-map",
  },
  {
    output: {
      filename: "./dist-commonjs.js",
      libraryTarget: "commonjs",
    },
    entry: "./a.js",
    mode: "production",
    target: "node",
  },
];
