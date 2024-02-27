module.exports = [
  {
    devtool: false,
    output: {
      filename: "./dist-first.js",
    },
    name: "first",
    entry: "./src/first.js",
    mode: "development",
    bail: true,
  },
  {
    devtool: false,
    output: {
      filename: "./dist-second.js",
    },
    name: "second",
    entry: "./src/second.js",
    mode: "development",
  },
];
