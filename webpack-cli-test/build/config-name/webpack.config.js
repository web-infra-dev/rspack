module.exports = [
  {
    output: {
      filename: "./dist-first.js",
    },
    name: "first",
    entry: "./src/first.js",
    mode: "development",
  },
  {
    output: {
      filename: "./dist-second.js",
    },
    name: "second",
    entry: "./src/second.js",
    mode: "development",
  },
  {
    output: {
      filename: "./dist-third.js",
    },
    name: "third",
    entry: "./src/third.js",
    mode: "none",
    stats: "verbose",
  },
];
