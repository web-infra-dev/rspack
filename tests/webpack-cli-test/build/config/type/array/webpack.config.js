module.exports = [
  {
    output: {
      filename: "./dist-amd.js",
      libraryTarget: "amd",
    },
    name: "amd",
    entry: "./a.js",
    mode: "development",
    stats: "verbose",
    devtool: "eval-cheap-module-source-map",
  },
  {
    output: {
      filename: "./dist-commonjs.js",
      libraryTarget: "commonjs",
    },
    name: "commonjs",
    entry: "./a.js",
    mode: "development",
    stats: "detailed",
    target: "node",
  },
];
