module.exports = {
  output: {
    filename: "./dist-commonjs.js",
    libraryTarget: "commonjs",
  },
  name: "commonjs",
  entry: "./init.js",
  mode: "development",
  target: "node",
};
